use std::sync::Arc;

use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryFilter, ColumnTrait, Set};

use crate::models::_entities::{
    sea_orm_active_enums::StrategyStatus,
    strategy,
};
use super::condition::{ConditionEvaluator, LuaEvaluator};
use super::drift::{DriftProvider, PerpAmount, PerpMarket, PositionSide};

const ENGINE_INTERVAL_SECS: u64 = 30;

/// Maps a DB symbol string (e.g. "SOLUSDT") to a PerpMarket variant.
fn symbol_to_perp_market(symbol: &str) -> Option<PerpMarket> {
    let s = symbol.to_uppercase();
    // Strip common suffixes
    let base = s.strip_suffix("USDT")
        .or_else(|| s.strip_suffix("USD"))
        .or_else(|| s.strip_suffix("PERP"))
        .unwrap_or(&s);
    match base {
        "SOL" => Some(PerpMarket::SOL),
        "BTC" => Some(PerpMarket::BTC),
        "ETH" => Some(PerpMarket::ETH),
        "APT" => Some(PerpMarket::APT),
        "BONK" => Some(PerpMarket::BONK),
        "DOGE" => Some(PerpMarket::DOGE),
        "BNB" => Some(PerpMarket::BNB),
        "SUI" => Some(PerpMarket::SUI),
        "PEPE" => Some(PerpMarket::PEPE),
        "XRP" => Some(PerpMarket::XRP),
        "LINK" => Some(PerpMarket::LINK),
        "AVAX" => Some(PerpMarket::AVAX),
        "ARB" => Some(PerpMarket::ARB),
        "OP" => Some(PerpMarket::OP),
        "SEI" => Some(PerpMarket::SEI),
        "INJ" => Some(PerpMarket::INJ),
        "RENDER" => Some(PerpMarket::RENDER),
        "PYTH" => Some(PerpMarket::PYTH),
        "TIA" => Some(PerpMarket::TIA),
        "JTO" => Some(PerpMarket::JTO),
        _ => None,
    }
}

fn side_to_position_side(side: &crate::models::_entities::sea_orm_active_enums::Side) -> PositionSide {
    match side {
        crate::models::_entities::sea_orm_active_enums::Side::Buy => PositionSide::Long,
        crate::models::_entities::sea_orm_active_enums::Side::Sell => PositionSide::Short,
    }
}

/// Starts the strategy engine background loop.
/// Call this once during app startup.
pub fn start(db: DatabaseConnection, drift: Arc<dyn DriftProvider>) {
    tokio::spawn(async move {
        tracing::info!("Strategy engine started (interval={ENGINE_INTERVAL_SECS}s)");
        let evaluator = LuaEvaluator;
        loop {
            if let Err(e) = tick(&db, &drift, &evaluator).await {
                tracing::error!("Strategy engine tick error: {e}");
            }
            tokio::time::sleep(std::time::Duration::from_secs(ENGINE_INTERVAL_SECS)).await;
        }
    });
}

async fn tick(
    db: &DatabaseConnection,
    drift: &Arc<dyn DriftProvider>,
    evaluator: &LuaEvaluator,
) -> Result<()> {
    check_approved_strategies(db, drift, evaluator).await?;
    check_stop_losses(db, drift).await?;
    Ok(())
}

/// For each strategy with status=approved: fetch market data, evaluate condition,
/// if met → open position via DriftProvider, set status to triggered.
async fn check_approved_strategies(
    db: &DatabaseConnection,
    drift: &Arc<dyn DriftProvider>,
    evaluator: &LuaEvaluator,
) -> Result<()> {
    let strategies = strategy::Entity::find()
        .filter(strategy::Column::Status.eq(StrategyStatus::Approved))
        .all(db)
        .await
        .map_err(Error::wrap)?;

    for strat in strategies {
        if let Err(e) = process_approved_strategy(db, drift, evaluator, &strat).await {
            tracing::error!("Error processing strategy #{}: {e}", strat.id);
            // Mark as failed
            let mut active: strategy::ActiveModel = strat.into();
            active.status = Set(StrategyStatus::Failed);
            let _ = active.update(db).await;
        }
    }

    Ok(())
}

async fn process_approved_strategy(
    db: &DatabaseConnection,
    drift: &Arc<dyn DriftProvider>,
    evaluator: &LuaEvaluator,
    strat: &strategy::Model,
) -> Result<()> {
    let market_data = drift.get_market_data(&strat.symbol).await?;

    // If condition is empty, treat as always-true (execute immediately)
    let condition_met = if strat.condition.is_empty() {
        true
    } else {
        evaluator.evaluate(&strat.condition, &market_data)?
    };

    if !condition_met {
        return Ok(());
    }

    tracing::info!(
        "Strategy #{} condition met for {} — executing trade",
        strat.id, strat.symbol
    );

    let perp_market = symbol_to_perp_market(&strat.symbol).ok_or_else(|| {
        Error::BadRequest(format!("unknown perp market for symbol: {}", strat.symbol))
    })?;

    let side = side_to_position_side(&strat.side);
    let quantity_f64 = strat.quantity.to_string().parse::<f64>().unwrap_or(0.0);

    let sig = drift
        .open_perp_position(perp_market, side, PerpAmount::ActualUnits(quantity_f64))
        .await?;

    tracing::info!("Strategy #{} trade executed, sig={sig}", strat.id);

    let mut active: strategy::ActiveModel = strat.clone().into();
    active.status = Set(StrategyStatus::Triggered);
    active.executed_at = Set(Some(chrono::Utc::now().into()));
    active.update(db).await.map_err(Error::wrap)?;

    Ok(())
}

/// For each triggered strategy with a stop_loss_pct: check current PnL,
/// if below threshold → close position, set status to stopped.
async fn check_stop_losses(
    db: &DatabaseConnection,
    drift: &Arc<dyn DriftProvider>,
) -> Result<()> {
    let strategies = strategy::Entity::find()
        .filter(strategy::Column::Status.eq(StrategyStatus::Triggered))
        .all(db)
        .await
        .map_err(Error::wrap)?;

    for strat in strategies {
        // Skip strategies without stop-loss configured
        let stop_loss_pct = match strat.stop_loss_pct {
            Some(pct) => {
                let f = pct.to_string().parse::<f64>().unwrap_or(0.0);
                if f == 0.0 { continue; }
                f
            }
            None => continue,
        };

        let perp_market = match symbol_to_perp_market(&strat.symbol) {
            Some(m) => m,
            None => continue,
        };

        match drift.get_position_pnl(perp_market).await {
            Ok(Some(pnl_pct)) => {
                // stop_loss_pct is stored as a positive number, e.g. 5.0 means -5% triggers stop
                if pnl_pct <= -stop_loss_pct {
                    tracing::warn!(
                        "Strategy #{} stop-loss triggered: PnL={pnl_pct:.2}% <= -{stop_loss_pct:.2}%",
                        strat.id
                    );

                    if let Err(e) = drift.close_perp_position(perp_market).await {
                        tracing::error!("Failed to close position for strategy #{}: {e}", strat.id);
                        continue;
                    }

                    let mut active: strategy::ActiveModel = strat.into();
                    active.status = Set(StrategyStatus::Stopped);
                    let _ = active.update(db).await;
                }
            }
            Ok(None) => {
                // Position no longer open — mark as stopped
                tracing::info!("Strategy #{} position no longer open, marking stopped", strat.id);
                let mut active: strategy::ActiveModel = strat.into();
                active.status = Set(StrategyStatus::Stopped);
                let _ = active.update(db).await;
            }
            Err(e) => {
                tracing::error!("Error checking PnL for strategy #{}: {e}", strat.id);
            }
        }
    }

    Ok(())
}
