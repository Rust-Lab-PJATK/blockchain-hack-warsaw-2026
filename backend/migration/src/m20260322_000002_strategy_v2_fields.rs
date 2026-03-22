use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Add Queued variant to existing strategy_status enum
        m.get_connection()
            .execute_unprepared("ALTER TYPE strategy_status ADD VALUE IF NOT EXISTS 'queued'")
            .await?;

        m.alter_table(
            Table::alter()
                .table(Strategy::Table)
                .add_column(
                    ColumnDef::new(Strategy::ScheduledAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .add_column(ColumnDef::new(Strategy::StopLossPrice).decimal().null())
                .add_column(
                    ColumnDef::new(Strategy::QueuedAt)
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
                .drop_column(Strategy::ScheduledAt)
                .drop_column(Strategy::StopLossPrice)
                .drop_column(Strategy::QueuedAt)
                .to_owned(),
        )
        .await
    }
}

#[derive(DeriveIden)]
enum Strategy {
    Table,
    ScheduledAt,
    StopLossPrice,
    QueuedAt,
}
