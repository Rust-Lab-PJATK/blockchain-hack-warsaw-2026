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

// --- Vercel AI Gateway Provider ---

use crate::services::mcp::TradingMcpServer;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use serde_json::Value;

pub struct VercelProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    gateway_url: String,
    mcp_server: Arc<TradingMcpServer>,
}

impl VercelProvider {
    pub fn new(mcp_server: Arc<TradingMcpServer>) -> Result<Self> {
        let api_key = std::env::var("AI_GATEWAY_API_KEY")
            .map_err(|_| loco_rs::Error::string("AI_GATEWAY_API_KEY env var not set"))?;

        let model = std::env::var("AI_GATEWAY_MODEL")
            .unwrap_or_else(|_| "anthropic/claude-sonnet-4-6".to_string());

        let gateway_url = std::env::var("AI_GATEWAY_URL")
            .unwrap_or_else(|_| "https://ai-gateway.vercel.sh/v1/chat/completions".to_string());

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
            model,
            gateway_url,
            mcp_server,
        })
    }

    fn tool_definitions() -> Value {
        serde_json::json!([
            {
                "type": "function",
                "function": {
                    "name": "list_symbols",
                    "description": "List all available trading symbols from the database.",
                    "parameters": { "type": "object", "properties": {}, "required": [] }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "list_condition_variables",
                    "description": "List all variables available in Lua condition expressions.",
                    "parameters": { "type": "object", "properties": {}, "required": [] }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "create_trade",
                    "description": "Create a trade order with a Lua condition expression.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "symbol": { "type": "string", "description": "Trading pair symbol, e.g. SOLUSDT" },
                            "side": { "type": "string", "enum": ["buy", "sell"] },
                            "order_type": { "type": "string", "enum": ["limit", "market", "stop_limit"] },
                            "quantity": { "type": "number" },
                            "leverage": { "type": "integer" },
                            "price": { "type": "number" },
                            "condition": { "type": "string", "description": "Lua expression, e.g. price < 129 and rsi < 30" }
                        },
                        "required": ["symbol", "side", "order_type", "quantity", "leverage", "price", "condition"]
                    }
                }
            }
        ])
    }

    async fn execute_tool(&self, name: &str, arguments: Value) -> Result<String> {
        let result: std::result::Result<CallToolResult, rmcp::ErrorData> = match name {
            "list_symbols" => self.mcp_server.call_list_symbols().await,
            "list_condition_variables" => self.mcp_server.call_list_condition_variables().await,
            "create_trade" => {
                let args = serde_json::from_value(arguments)
                    .map_err(|e| loco_rs::Error::string(&format!("Invalid tool args: {e}")))?;
                self.mcp_server.call_create_trade(Parameters(args)).await
            }
            _ => return Err(loco_rs::Error::string(&format!("Unknown tool: {name}"))),
        };

        match result {
            Ok(r) => {
                let texts: Vec<String> = r
                    .content
                    .iter()
                    .filter_map(|c| c.raw.as_text().map(|t| t.text.clone()))
                    .collect();
                Ok(texts.join("\n"))
            }
            Err(e) => Ok(format!("Tool error: {}", e.message)),
        }
    }

    async fn call_gateway(&self, messages: &[Value]) -> Result<Value> {
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "tools": Self::tool_definitions(),
            "tool_choice": "auto"
        });

        let res = self
            .client
            .post(&self.gateway_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| loco_rs::Error::string(&format!("Gateway request failed: {e}")))?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(loco_rs::Error::string(&format!(
                "Gateway returned {status}: {body}"
            )));
        }

        let json: Value = res
            .json()
            .await
            .map_err(|e| loco_rs::Error::string(&format!("Failed to parse gateway response: {e}")))?;
        Ok(json)
    }
}

const MAX_TOOL_ROUNDS: usize = 10;

#[async_trait]
impl LlmProvider for VercelProvider {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse> {
        let mut conversation: Vec<Value> = messages
            .iter()
            .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
            .collect();

        for _ in 0..MAX_TOOL_ROUNDS {
            let response = self.call_gateway(&conversation).await?;

            let choice = response["choices"]
                .get(0)
                .ok_or_else(|| loco_rs::Error::string("No choices in gateway response"))?;

            let message = &choice["message"];
            let finish_reason = choice["finish_reason"].as_str().unwrap_or("");

            // If model wants to call tools
            if finish_reason == "tool_calls" || message.get("tool_calls").is_some() {
                let tool_calls = message["tool_calls"]
                    .as_array()
                    .ok_or_else(|| loco_rs::Error::string("Invalid tool_calls format"))?;

                // Add assistant message with tool calls to conversation
                conversation.push(message.clone());

                // Execute each tool and add results
                for tc in tool_calls {
                    let id = tc["id"].as_str().unwrap_or("");
                    let name = tc["function"]["name"].as_str().unwrap_or("");
                    let args: Value = tc["function"]["arguments"]
                        .as_str()
                        .and_then(|s| serde_json::from_str(s).ok())
                        .unwrap_or(Value::Object(Default::default()));

                    let result = self.execute_tool(name, args).await?;

                    conversation.push(serde_json::json!({
                        "role": "tool",
                        "tool_call_id": id,
                        "content": result
                    }));
                }

                continue;
            }

            // Final text response
            let content = message["content"]
                .as_str()
                .unwrap_or("")
                .to_string();

            return Ok(ChatResponse { content });
        }

        Err(loco_rs::Error::string(
            "Max tool call rounds exceeded",
        ))
    }
}
