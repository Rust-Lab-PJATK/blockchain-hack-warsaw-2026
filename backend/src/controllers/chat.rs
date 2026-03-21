use axum::Json;
use loco_rs::prelude::*;
use serde::Deserialize;
use std::sync::Arc;

use crate::services::llm::{ChatMessage, LlmProvider};

#[derive(Debug, Deserialize)]
struct ChatRequest {
    messages: Vec<ChatMessage>,
}

#[debug_handler]
async fn chat(
    axum::Extension(_provider): axum::Extension<Arc<dyn LlmProvider>>,
    Json(payload): Json<ChatRequest>,
) -> Result<Response> {
    let response = _provider.chat(&payload.messages).await?;
    format::json(response)
}

pub fn routes() -> Routes {
    Routes::new().prefix("/api/chat").add("/", post(chat))
}
