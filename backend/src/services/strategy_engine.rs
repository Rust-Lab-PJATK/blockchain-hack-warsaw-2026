use std::sync::Arc;
use crate::controllers::notification::ClientMap;
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryFilter, ColumnTrait, Set, Condition};

use crate::models::_entities::{
    chat_message,
    sea_orm_active_enums::StrategyStatus,
    strategy,
};
use super::condition::{ConditionEvaluator, LuaEvaluator};
use super::drift::{DriftProvider, PerpAmount, PerpMarket, PositionSide};

const ENGINE_INTERVAL_SECS: u64 = 30;
const MARKET_DATA_RETRY_COUNT: u32 = 3;
const QUEUE_TIMEOUT_SECS: i64 = 3600; // 1 hour

/// Maps a DB symbol string (e.g. "SOLUSDT") to a PerpMarket variant.
fn symbol_to_perp_market(symbol: &str) -> Option<PerpMarket> {
    let s = symbol.to_uppercase();
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
// FIX 1: client_id must be String (owned), not str (unsized) — str alone can never be a function parameter
pub fn start(db: DatabaseConnection, drift: Arc<dyn DriftProvider>, clients: ClientMap, client_id: String) {
    tokio::spawn(async move {
        tracing::info!("Strategy engine started (interval={ENGINE_INTERVAL_SECS}s)");
        let evaluator = LuaEvaluator;
        loop {
            // FIX 2: pass clients and client_id down to tick
            if let Err(e) = tick(&db, &drift, &evaluator, &clients, &client_id).await {
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
    clients: &ClientMap,
    client_id: &str,
) -> Result<()> {
    // FIX 3: pass clients and client_id into all three check functions
    check_approved_strategies(db, drift, evaluator, clients, client_id).await?;
    check_queued_strategies(db, drift, clients, client_id).await?;
    check_stop_losses(db, drift, clients, client_id).await?;
    Ok(())
}

// ── Fetch market data with retry ───────────────────────────────────────────

async fn fetch_market_data_with_retry(
    drift: &Arc<dyn DriftProvider>,
    symbol: &str,
) -> Result<std::collections::HashMap<String, f64>> {
    let mut last_err = None;
    for attempt in 1..=MARKET_DATA_RETRY_COUNT {
        match drift.get_market_data(symbol).await {
            Ok(data) => return Ok(data),
            Err(e) => {
                tracing::warn!(
                    "Market data fetch attempt {attempt}/{MARKET_DATA_RETRY_COUNT} failed for {symbol}: {e}"
                );
                last_err = Some(e);
                if attempt < MARKET_DATA_RETRY_COUNT {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }
    }
    Err(last_err.unwrap_or_else(|| Error::string("market data fetch failed")))
}

// ── Save notification to chat_messages ─────────────────────────────────────

async fn save_notification(
    db: &DatabaseConnection,
    message: &str,
    clients: &ClientMap,
    client_id: &str,
) {
    // 1. Persist to DB
    let record = chat_message::ActiveModel {
        user_message: Set(String::new()),
        assistant_message: Set(message.to_owned()),
        created_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };
    if let Err(e) = record.insert(db).await {
        tracing::error!("Failed to save notification: {e}");
    }

    // 2. Push to SSE client if connected
    publish_sse(clients, client_id, message).await;
}

async fn publish_sse(clients: &ClientMap, client_id: &str, message: &str) {
    let map = clients.lock().await;
    if let Some(tx) = map.get(client_id) {
        let payload = serde_json::json!({ "message": message }).to_string();
        if tx.send(payload).await.is_err() {
            tracing::warn!("Client {client_id} disconnected, notification saved to DB only");
        }
    }
}

// ── Phase 1: Check approved strategies ─────────────────────────────────────

async fn check_approved_strategies(
    db: &DatabaseConnection,
    drift: &Arc<dyn DriftProvider>,
    evaluator: &LuaEvaluator,
    clients: &ClientMap,
    client_id: &str,
) -> Result<()> {
    let strategies = strategy::Entity::find()
        .filter(strategy::Column::Status.eq(StrategyStatus::Approved))
        .all(db)
        .await
        .map_err(Error::wrap)?;

    for strat in strategies {
        if let Err(e) = process_approved_strategy(db, drift, evaluator, &strat, clients, client_id).await {
            tracing::error!("Error processing strategy #{}: {e}", strat.id);
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
    clients: &ClientMap,
    client_id: &str,
) -> Result<()> {
    // 1. Check scheduled_at — skip if time hasn't arrived yet
    if let Some(scheduled_at) = strat.scheduled_at {
        let now = chrono::Utc::now();
        if now < scheduled_at {
            return Ok(());
        }
    }

    // 2. Fetch market data with retry (3x)
    let market_data = match fetch_market_data_with_retry(drift, &strat.symbol).await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!(
                "Strategy #{}: market data unavailable after {MARKET_DATA_RETRY_COUNT} retries: {e}",
                strat.id
            );
            save_notification(
                db,
                &format!(
                    "Strategy #{} ({}): market data feed unavailable, skipping this tick.",
                    strat.id, strat.symbol
                ),
                clients,
                client_id,
            ).await;
            return Ok(()); // Don't fail the strategy, try again next tick
        }
    };

    // 3. Evaluate Lua condition
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

    // 4. Check user balance before executing
    let balance = drift.get_user_balance().await?;
    let required = strat.quantity.to_string().parse::<f64>().unwrap_or(0.0)
        * strat.price.to_string().parse::<f64>().unwrap_or(0.0)
        / strat.leverage.max(1) as f64;

    if balance < required {
        tracing::warn!(
            "Strategy #{}: insufficient balance ({balance:.2} < {required:.2}), skipping",
            strat.id
        );
        save_notification(
            db,
            &format!(
                "Strategy #{} ({}): insufficient funds. Available: ${balance:.2}, required: ~${required:.2}. Trade not executed.",
                strat.id, strat.symbol
            ),
            clients,
            client_id,
        ).await;
        return Ok(()); // Don't fail — user might deposit funds
    }

    // 5. Execute trade via DriftProvider
    let perp_market = symbol_to_perp_market(&strat.symbol).ok_or_else(|| {
        Error::BadRequest(format!("unknown perp market for symbol: {}", strat.symbol))
    })?;

    let side = side_to_position_side(&strat.side);
    let quantity_f64 = strat.quantity.to_string().parse::<f64>().unwrap_or(0.0);
    let current_price = market_data.get("price").copied().unwrap_or(0.0);

    match drift
        .open_perp_position(perp_market, side, PerpAmount::ActualUnits(quantity_f64))
        .await
    {
        Ok(sig) => {
            tracing::info!("Strategy #{} trade executed, sig={sig}", strat.id);

            let mut active: strategy::ActiveModel = strat.clone().into();
            active.status = Set(StrategyStatus::Triggered);
            active.executed_at = Set(Some(chrono::Utc::now().into()));
            active.update(db).await.map_err(Error::wrap)?;

            // 6. Save notification with AI reasoning
            save_notification(
                db,
                &format!(
                    "Executed trade for strategy #{}: {:?} {} {} at price ${current_price:.2}, because condition \"{}\" was met. Tx: {sig}",
                    strat.id, strat.side, quantity_f64, strat.symbol, strat.condition
                ),
                clients,
                client_id,
            ).await;
        }
        Err(e) => {
            // Drift downtime: queue the trade
            tracing::warn!(
                "Strategy #{}: trade execution failed ({e}), queuing for retry",
                strat.id
            );

            let mut active: strategy::ActiveModel = strat.clone().into();
            active.status = Set(StrategyStatus::Queued);
            active.queued_at = Set(Some(chrono::Utc::now().into()));
            active.update(db).await.map_err(Error::wrap)?;

            save_notification(
                db,
                &format!(
                    "Strategy #{} ({}): trade execution failed ({}). Queued for automatic retry.",
                    strat.id, strat.symbol, e
                ),
                clients,
                client_id,
            ).await;
        }
    }

    Ok(())
}

// ── Phase 2: Check queued strategies (Drift downtime handling) ─────────────

async fn check_queued_strategies(
    db: &DatabaseConnection,
    drift: &Arc<dyn DriftProvider>,
    clients: &ClientMap,
    client_id: &str,
) -> Result<()> {
    let strategies = strategy::Entity::find()
        .filter(strategy::Column::Status.eq(StrategyStatus::Queued))
        .all(db)
        .await
        .map_err(Error::wrap)?;

    let now = chrono::Utc::now();

    for strat in strategies {
        // Cancel if queued for more than 1 hour
        if let Some(queued_at) = strat.queued_at {
            let elapsed = now.signed_duration_since(queued_at);
            if elapsed.num_seconds() > QUEUE_TIMEOUT_SECS {
                tracing::warn!(
                    "Strategy #{}: queued for >1h, cancelling",
                    strat.id
                );
                let mut active: strategy::ActiveModel = strat.clone().into();
                active.status = Set(StrategyStatus::Failed);
                let _ = active.update(db).await;

                save_notification(
                    db,
                    &format!(
                        "Strategy #{} ({}): trade cancelled — queued for over 1 hour without successful execution.",
                        strat.id, strat.symbol
                    ),
                    clients,
                    client_id,
                ).await;
                continue;
            }
        }

        // Retry the trade
        let perp_market = match symbol_to_perp_market(&strat.symbol) {
            Some(m) => m,
            None => continue,
        };

        let side = side_to_position_side(&strat.side);
        let quantity_f64 = strat.quantity.to_string().parse::<f64>().unwrap_or(0.0);

        match drift
            .open_perp_position(perp_market, side, PerpAmount::ActualUnits(quantity_f64))
            .await
        {
            Ok(sig) => {
                tracing::info!("Strategy #{} queued trade executed, sig={sig}", strat.id);

                let mut active: strategy::ActiveModel = strat.clone().into();
                active.status = Set(StrategyStatus::Triggered);
                active.executed_at = Set(Some(chrono::Utc::now().into()));
                active.queued_at = Set(None);
                active.update(db).await.map_err(Error::wrap)?;

                save_notification(
                    db,
                    &format!(
                        "Strategy #{} ({}): queued trade successfully executed. Tx: {sig}",
                        strat.id, strat.symbol
                    ),
                    clients,
                    client_id,
                ).await;
            }
            Err(e) => {
                tracing::warn!(
                    "Strategy #{}: queued trade retry failed: {e}",
                    strat.id
                );
                // Stay queued, will retry next tick
            }
        }
    }

    Ok(())
}

// ── Phase 3: Stop-loss monitoring ──────────────────────────────────────────

async fn check_stop_losses(
    db: &DatabaseConnection,
    drift: &Arc<dyn DriftProvider>,
    clients: &ClientMap,
    client_id: &str,
) -> Result<()> {
    let strategies = strategy::Entity::find()
        .filter(strategy::Column::Status.eq(StrategyStatus::Triggered))
        .filter(
            Condition::any()
                .add(strategy::Column::StopLossPct.is_not_null())
                .add(strategy::Column::StopLossPrice.is_not_null()),
        )
        .all(db)
        .await
        .map_err(Error::wrap)?;

    for strat in strategies {
        let perp_market = match symbol_to_perp_market(&strat.symbol) {
            Some(m) => m,
            None => continue,
        };

        // Check absolute stop-loss price
        if let Some(sl_price) = &strat.stop_loss_price {
            let sl_price_f64 = sl_price.to_string().parse::<f64>().unwrap_or(0.0);
            if sl_price_f64 > 0.0 {
                match drift.get_current_price(perp_market).await {
                    Ok(current_price) => {
                        let is_long = strat.side == crate::models::_entities::sea_orm_active_enums::Side::Buy;
                        let stop_hit = if is_long {
                            current_price <= sl_price_f64
                        } else {
                            current_price >= sl_price_f64
                        };

                        if stop_hit {
                            tracing::warn!(
                                "Strategy #{} stop-loss price hit: current={current_price:.2}, stop={sl_price_f64:.2}",
                                strat.id
                            );
                            execute_stop_loss(db, drift, perp_market, &strat, &format!(
                                "Closed position for strategy #{} ({}): price ${current_price:.2} hit stop-loss at ${sl_price_f64:.2}.",
                                strat.id, strat.symbol
                            ), clients, client_id).await;
                            continue;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error fetching price for strategy #{}: {e}", strat.id);
                    }
                }
            }
        }

        // Check percentage-based stop-loss
        if let Some(sl_pct) = &strat.stop_loss_pct {
            let sl_pct_f64 = sl_pct.to_string().parse::<f64>().unwrap_or(0.0);
            if sl_pct_f64 > 0.0 {
                match drift.get_position_pnl(perp_market).await {
                    Ok(Some(pnl_pct)) => {
                        if pnl_pct <= -sl_pct_f64 {
                            tracing::warn!(
                                "Strategy #{} stop-loss % triggered: PnL={pnl_pct:.2}% <= -{sl_pct_f64:.2}%",
                                strat.id
                            );
                            execute_stop_loss(db, drift, perp_market, &strat, &format!(
                                "Closed position for strategy #{} ({}): PnL {pnl_pct:.2}% exceeded stop-loss threshold of -{sl_pct_f64:.2}%.",
                                strat.id, strat.symbol
                            ), clients, client_id).await;
                        }
                    }
                    Ok(None) => {
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
        }
    }

    Ok(())
}

async fn execute_stop_loss(
    db: &DatabaseConnection,
    drift: &Arc<dyn DriftProvider>,
    market: PerpMarket,
    strat: &strategy::Model,
    notification_msg: &str,
    clients: &ClientMap,
    client_id: &str,
) {
    match drift.close_perp_position(market).await {
        Ok(sig) => {
            tracing::info!("Strategy #{} stop-loss executed, sig={sig}", strat.id);
            let mut active: strategy::ActiveModel = strat.clone().into();
            active.status = Set(StrategyStatus::Stopped);
            let _ = active.update(db).await;

            save_notification(db, &format!("{notification_msg} Tx: {sig}"), clients, client_id).await;
        }
        Err(e) => {
            tracing::error!("Failed to close position for strategy #{}: {e}", strat.id);
        }
    }
}