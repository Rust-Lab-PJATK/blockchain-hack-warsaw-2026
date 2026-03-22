  use sea_orm::ActiveValue::Set;
  use crate::models::_entities::strategy::{ActiveModel, Model};
  use crate::models::_entities::symbol;
  use loco_rs::prelude::*;
  use crate::models::_entities::sea_orm_active_enums::{OrderType, Side};

  impl Model {
      pub async fn create(
          db: &DatabaseConnection,
          symbol: &str,
          side: Side,
          order_type: OrderType,
          leverage: i32,
          price: Decimal,
          quantity: Decimal,
      ) -> ModelResult<Self> {
          let symbol_exists = symbol::Entity::find()
              .filter(symbol::Column::Name.eq(symbol))
              .one(db)
              .await?
              .is_some();

          if !symbol_exists {
              return Err(ModelError::Message(format!("Could not create strategy - Symbol '{}' does not exist", symbol)));
          }

          let strategy = ActiveModel {
              symbol: Set(symbol.to_owned()),
              side: Set(side),
              order_type: Set(order_type),
              leverage: Set(leverage),
              price: Set(price),
              quantity: Set(quantity),
              ..Default::default()
          };

          Ok(strategy.insert(db).await?)
      }
  }
