use sea_orm::ActiveValue::Set;
use crate::models::_entities::symbol::{self, ActiveModel, Entity as Symbol, Model};
use loco_rs::prelude::*;

impl Model {
    pub async fn create(db: &DatabaseConnection, name: &str) -> ModelResult<Self> {
        let symbol = ActiveModel {
            name: Set(name.to_owned()),
            ..Default::default()
        };
        Ok(symbol.insert(db).await?)
    }

    pub async fn exists_by_name(db: &DatabaseConnection, name: &str) -> ModelResult<bool> {
        let symbol = Symbol::find()
            .filter(symbol::Column::Name.eq(name))
            .one(db)
            .await?;
        Ok(symbol.is_some())
    }
}