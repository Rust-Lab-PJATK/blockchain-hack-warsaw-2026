#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;

mod m20260321_143849_strategies;
mod m20260321_160104_symbols;
mod m20260321_170000_chat_messages;
mod m20260322_000001_strategy_engine_fields;
mod m20260322_000002_strategy_v2_fields;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260321_143849_strategies::Migration),
            Box::new(m20260321_160104_symbols::Migration),
            Box::new(m20260321_170000_chat_messages::Migration),
            Box::new(m20260322_000001_strategy_engine_fields::Migration),
            Box::new(m20260322_000002_strategy_v2_fields::Migration),
        ]
    }
}
