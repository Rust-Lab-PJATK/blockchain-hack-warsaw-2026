use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_type(
            Type::create()
                .as_enum(StrategyStatus::Type)
                .values([
                    StrategyStatus::Waiting,
                    StrategyStatus::Approved,
                    StrategyStatus::Triggered,
                    StrategyStatus::Stopped,
                    StrategyStatus::Failed,
                ])
                .to_owned(),
        )
        .await?;

        m.alter_table(
            Table::alter()
                .table(Strategy::Table)
                .add_column(
                    ColumnDef::new(Strategy::Status)
                        .custom(StrategyStatus::Type)
                        .not_null()
                        .default("waiting"),
                )
                .add_column(ColumnDef::new(Strategy::Condition).text().not_null().default(""))
                .add_column(ColumnDef::new(Strategy::StopLossPct).decimal().null())
                .add_column(
                    ColumnDef::new(Strategy::ExecutedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Strategy::Table)
                .drop_column(Strategy::Status)
                .drop_column(Strategy::Condition)
                .drop_column(Strategy::StopLossPct)
                .drop_column(Strategy::ExecutedAt)
                .to_owned(),
        )
        .await?;

        m.drop_type(Type::drop().name(StrategyStatus::Type).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Strategy {
    Table,
    Status,
    Condition,
    StopLossPct,
    ExecutedAt,
}

#[derive(DeriveIden)]
enum StrategyStatus {
    #[sea_orm(iden = "strategy_status")]
    Type,
    Waiting,
    Approved,
    Triggered,
    Stopped,
    Failed,
}
