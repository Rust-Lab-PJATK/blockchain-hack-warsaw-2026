use sea_orm_migration::{prelude::*, schema::*};
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_type(
            Type::create()
                .as_enum(Side::Type)
                .values([Side::Buy, Side::Sell])
                .to_owned(),
        )
            .await?;

        m.create_type(
            Type::create()
                .as_enum(OrderType::Type)
                .values([OrderType::Limit, OrderType::Market, OrderType::StopLimit])
                .to_owned(),
        )
            .await?;

        m.create_table(
            Table::create()
                .table(Strategy::Table)
                .if_not_exists()
                .col(pk_auto(Strategy::Id).big_integer())
                .col(ColumnDef::new(Strategy::Symbol).string().not_null())
                .col(ColumnDef::new(Strategy::Side).custom(Side::Type).not_null())
                .col(ColumnDef::new(Strategy::OrderType).custom(OrderType::Type).not_null())
                .col(ColumnDef::new(Strategy::Leverage).integer().not_null())
                .col(ColumnDef::new(Strategy::Price).decimal().not_null())
                .col(ColumnDef::new(Strategy::Quantity).decimal().not_null())
                .to_owned(),
        )
            .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(Strategy::Table).to_owned()).await?;
        m.drop_type(Type::drop().name(Side::Type).to_owned()).await?;
        m.drop_type(Type::drop().name(OrderType::Type).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Strategy {
    Table,
    Id,
    Symbol,
    Side,
    OrderType,
    Leverage,
    Price,
    Quantity,
}

#[derive(DeriveIden)]
enum Side {
    #[sea_orm(iden = "side")]
    Type,
    Buy,
    Sell,
}

#[derive(DeriveIden)]
enum OrderType {
    #[sea_orm(iden = "order_type")]
    Type,
    Limit,
    Market,
    StopLimit,
}