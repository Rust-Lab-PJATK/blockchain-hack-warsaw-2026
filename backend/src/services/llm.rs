use async_trait::async_trait;
use axum::Extension;
use loco_rs::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse>;
}

pub type LlmProviderExtension = Extension<Arc<dyn LlmProvider>>;

pub struct MockProvider;

#[async_trait]
impl LlmProvider for MockProvider {
    async fn chat(&self, _messages: &[ChatMessage]) -> Result<ChatResponse> {
        Ok(ChatResponse {
            content: "LLM provider not configured.".into(),
        })
    }
}
