use loco_rs::prelude::*;
use serde::Deserialize;
use rust_decimal::Decimal;
use crate::models::_entities::strategy;
use crate::models::_entities::sea_orm_active_enums::{Side, OrderType, Status};

#[derive(Deserialize)]
pub struct CreateStrategyParams {
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub leverage: i32,
    pub price: Decimal,
    pub quantity: Decimal,
    pub status: Status
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
        params.status
    ).await {
        Ok(strategy) => format::json(strategy),
        Err(ModelError::Message(msg)) => Err(Error::BadRequest(msg)),
        Err(_) => Err(Error::InternalServerError),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/strategies")
        .add("/", post(create))
}