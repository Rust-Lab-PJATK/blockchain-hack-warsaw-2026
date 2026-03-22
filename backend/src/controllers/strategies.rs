use axum::extract::Path;
use loco_rs::prelude::*;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::models::_entities::sea_orm_active_enums::{OrderType, Side};
use crate::models::_entities::strategy;
use crate::services::drift::MARKET_DATA_VARIABLES;

#[derive(Deserialize)]
pub struct CreateStrategyParams {
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub leverage: i32,
    pub price: Decimal,
    pub quantity: Decimal,
    #[serde(default)]
    pub condition: String,
    pub stop_loss_pct: Option<Decimal>,
}

async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<CreateStrategyParams>,
) -> Result<Response> {
    match strategy::Model::create(
        &ctx.db,
        &params.symbol,
        params.side,
        params.order_type,
        params.leverage,
        params.price,
        params.quantity,
        &params.condition,
        params.stop_loss_pct,
    )
    .await
    {
        Ok(strategy) => format::json(strategy),
        Err(ModelError::Message(msg)) => Err(Error::BadRequest(msg)),
        Err(_) => Err(Error::InternalServerError),
    }
}

async fn approve(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Response> {
    match strategy::Model::approve(&ctx.db, id).await {
        Ok(strategy) => format::json(strategy),
        Err(ModelError::Message(msg)) => Err(Error::BadRequest(msg)),
        Err(_) => Err(Error::InternalServerError),
    }
}

async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    match strategy::Model::list_all(&ctx.db).await {
        Ok(strategies) => format::json(strategies),
        Err(_) => Err(Error::InternalServerError),
    }
}

async fn condition_variables() -> Result<Response> {
    let vars: Vec<serde_json::Value> = MARKET_DATA_VARIABLES
        .iter()
        .map(|(name, description)| {
            serde_json::json!({
                "name": name,
                "type": "number",
                "description": description,
            })
        })
        .collect();
    format::json(vars)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/strategies")
        .add("/", post(create))
        .add("/", get(list))
        .add("/{id}/approve", post(approve))
        .add("/condition-variables", get(condition_variables))
}
