use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(Symbol::Table)
                .if_not_exists()
                .col(pk_auto(Symbol::Id).big_integer())
                .col(ColumnDef::new(Symbol::Name).string().not_null().unique_key())
                .to_owned(),
        ).await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(Symbol::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Symbol {
    Table,
    Id,
    Name
}