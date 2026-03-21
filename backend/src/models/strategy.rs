use async_trait::async_trait;
use sea_orm::{entity::prelude::*, ActiveValue, QueryFilter};

use super::_entities::{strategy, symbol};

#[async_trait]
impl ActiveModelBehavior for strategy::ActiveModel {
    async fn before_save<C>(self, db: &C, _insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if let ActiveValue::Set(ref sym) = self.symbol {
            let exists = symbol::Entity::find()
                .filter(symbol::Column::Name.eq(sym.clone()))
                .one(db)
                .await?;

            if exists.is_none() {
                return Err(DbErr::Custom(format!(
                    "Symbol '{}' does not exist in the symbols table",
                    sym
                )));
            }
        }

        Ok(self)
    }
}
