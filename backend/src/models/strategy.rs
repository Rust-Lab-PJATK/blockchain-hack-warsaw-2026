  use sea_orm::ActiveValue::Set;
  use crate::models::_entities::strategy::{ActiveModel, Entity, Model};
  use crate::models::_entities::symbol;
  use crate::models::_entities::sea_orm_active_enums::{OrderType, Side, StrategyStatus};
  use loco_rs::prelude::*;

  impl Model {
      #[allow(clippy::too_many_arguments)]
      pub async fn create(
          db: &DatabaseConnection,
          symbol: &str,
          side: Side,
          order_type: OrderType,
          leverage: i32,
          price: Decimal,
          quantity: Decimal,
          condition: &str,
          stop_loss_pct: Option<Decimal>,
          stop_loss_price: Option<Decimal>,
          scheduled_at: Option<chrono::DateTime<chrono::FixedOffset>>,
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
              condition: Set(condition.to_owned()),
              stop_loss_pct: Set(stop_loss_pct),
              stop_loss_price: Set(stop_loss_price),
              scheduled_at: Set(scheduled_at.map(|dt| dt.into())),
              status: Set(StrategyStatus::Waiting),
              ..Default::default()
          };

          Ok(strategy.insert(db).await?)
      }

      pub async fn approve(db: &DatabaseConnection, id: i64) -> ModelResult<Self> {
          let strat = Entity::find_by_id(id)
              .one(db)
              .await?
              .ok_or_else(|| ModelError::Message(format!("Strategy #{id} not found")))?;

          if strat.status != StrategyStatus::Waiting {
              return Err(ModelError::Message(format!(
                  "Strategy #{id} cannot be approved — current status: {:?}",
                  strat.status
              )));
          }

          let mut active: ActiveModel = strat.into();
          active.status = Set(StrategyStatus::Approved);
          Ok(active.update(db).await?)
      }

      pub async fn list_all(db: &DatabaseConnection) -> ModelResult<Vec<Self>> {
          Ok(Entity::find().all(db).await?)
      }
  }
