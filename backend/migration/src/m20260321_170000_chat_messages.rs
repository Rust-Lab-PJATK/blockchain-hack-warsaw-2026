use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ChatMessage::Table)
                    .if_not_exists()
                    .col(pk_auto(ChatMessage::Id).big_integer())
                    .col(text(ChatMessage::UserMessage))
                    .col(text(ChatMessage::AssistantMessage))
                    .col(
                        timestamp_with_time_zone(ChatMessage::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChatMessage::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ChatMessage {
    Table,
    Id,
    UserMessage,
    AssistantMessage,
    CreatedAt,
}
