use axum::{extract::Query, Json};
use chrono::{DateTime, FixedOffset};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::models::_entities::chat_message;
use crate::services::llm::{ChatMessage, LlmProvider};

#[derive(Debug, Serialize)]
struct ChatHistoryResponse {
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Deserialize)]
struct ChatRequest {
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Deserialize)]
struct ChatHistoryQuery {
    from: DateTime<FixedOffset>,
    to: DateTime<FixedOffset>,
}

#[debug_handler]
async fn chat(
    State(ctx): State<AppContext>,
    axum::Extension(provider): axum::Extension<Arc<dyn LlmProvider>>,
    Json(payload): Json<ChatRequest>,
) -> Result<Response> {
    let user_message = payload
        .messages
        .last()
        .map(|m| m.content.clone())
        .unwrap_or_default();

    let response = provider.chat(&payload.messages).await?;

    let record = chat_message::ActiveModel {
        user_message: Set(user_message),
        assistant_message: Set(response.content.clone()),
        ..Default::default()
    };
    record
        .insert(&ctx.db)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;

    format::json(response)
}

#[debug_handler]
async fn history(
    State(ctx): State<AppContext>,
    Query(params): Query<ChatHistoryQuery>,
) -> Result<Response> {
    let records = chat_message::Entity::find()
        .filter(chat_message::Column::CreatedAt.gte(params.from))
        .filter(chat_message::Column::CreatedAt.lte(params.to))
        .order_by_asc(chat_message::Column::CreatedAt)
        .all(&ctx.db)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;

    let mut messages = Vec::new();
    for record in records {
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: record.user_message,
        });
        messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: record.assistant_message,
        });
    }

    format::json(ChatHistoryResponse { messages })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/chat")
        .add("/", post(chat))
        .add("/history", get(history))
}
