use rmcp::{
    ServerHandler,
    handler::server::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::*,
    tool, tool_handler, tool_router,
    ErrorData as McpError,
};
use serde::Deserialize;

use crate::models::_entities::{
    sea_orm_active_enums::{OrderType, Side, StrategyStatus},
    strategy,
    symbol,
};
use crate::services::drift::MARKET_DATA_VARIABLES;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::sync::Arc;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateTradeArgs {
    #[schemars(description = "Trading pair symbol from the symbols table, e.g. SOLUSDT, ETHUSDT, BTCUSDT")]
    pub symbol: String,

    #[schemars(description = "Order side")]
    pub side: Side,

    #[schemars(description = "Order type")]
    pub order_type: OrderType,

    #[schemars(description = "Order quantity")]
    pub quantity: f64,

    #[schemars(description = "Leverage multiplier")]
    pub leverage: i32,

    #[schemars(description = "Order price (for LIMIT orders)")]
    pub price: f64,

    #[schemars(
        description = "Lua expression evaluated against market variables (price, volume, etc.). Must return boolean. Example: price < 129. Use list_condition_variables to discover available variables."
    )]
    pub condition: String,

    #[schemars(description = "Optional stop-loss percentage (positive number). If the position PnL drops below -X%, it will be automatically closed. Example: 5.0 means close at -5%")]
    pub stop_loss_pct: Option<f64>,

    #[schemars(description = "Optional absolute stop-loss price. For long positions: if price drops below this value, close. For short: if price rises above this value, close. Example: 120.0")]
    pub stop_loss_price: Option<f64>,

    #[schemars(description = "Optional scheduled execution time in ISO 8601 format. The strategy will only execute after this time. Example: 2026-03-22T18:00:00Z")]
    pub scheduled_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TradingMcpServer {
    db: Arc<DatabaseConnection>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl TradingMcpServer {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db: Arc::new(db),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "List all variables available in Lua condition expressions. Returns variable names, types, and descriptions. Use this to understand what can be used in the 'condition' field of create_trade.")]
    async fn list_condition_variables(
        &self,
    ) -> Result<CallToolResult, McpError> {
        let vars: Vec<serde_json::Value> = MARKET_DATA_VARIABLES
            .iter()
            .map(|(name, desc)| {
                serde_json::json!({
                    "name": name,
                    "type": "number",
                    "description": desc,
                })
            })
            .collect();

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&vars)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?,
        )]))
    }

    #[tool(description = "List all available trading symbols from the database.")]
    async fn list_symbols(&self) -> Result<CallToolResult, McpError> {
        let symbols = symbol::Entity::find()
            .all(self.db.as_ref())
            .await
            .map_err(|e| McpError::internal_error(format!("DB query failed: {e}"), None))?;

        let names: Vec<&str> = symbols
            .iter()
            .map(|s| s.name.as_str())
            .collect();

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&names)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?,
        )]))
    }

    #[tool(description = "Create a trade order. The condition field is a Lua expression evaluated against market variables. The trade will be created with status 'waiting' and must be approved by the user before the strategy engine will execute it.")]
    async fn create_trade(
        &self,
        Parameters(args): Parameters<CreateTradeArgs>,
    ) -> Result<CallToolResult, McpError> {
        let stop_loss_pct = args
            .stop_loss_pct
            .map(rust_decimal::Decimal::try_from)
            .transpose()
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let stop_loss_price = args
            .stop_loss_price
            .map(rust_decimal::Decimal::try_from)
            .transpose()
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let scheduled_at = args
            .scheduled_at
            .as_deref()
            .map(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.with_timezone(&chrono::Utc).into())
                    .map_err(|e| McpError::invalid_params(format!("invalid scheduled_at: {e}"), None))
            })
            .transpose()?;

        let record = strategy::ActiveModel {
            symbol: Set(args.symbol.clone()),
            side: Set(args.side.clone()),
            order_type: Set(args.order_type.clone()),
            quantity: Set(rust_decimal::Decimal::try_from(args.quantity)
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?),
            leverage: Set(args.leverage),
            price: Set(rust_decimal::Decimal::try_from(args.price)
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?),
            condition: Set(args.condition.clone()),
            stop_loss_pct: Set(stop_loss_pct),
            stop_loss_price: Set(stop_loss_price),
            scheduled_at: Set(scheduled_at),
            status: Set(StrategyStatus::Waiting),
            ..Default::default()
        };

        let result = record.insert(self.db.as_ref()).await.map_err(|e| {
            McpError::internal_error(format!("DB insert failed: {e}"), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Strategy #{} created (status: waiting — user must approve before execution): {:?} {} {} @ {} x{} (type: {:?}, condition: \"{}\", stop_loss_pct: {:?}, stop_loss_price: {:?}, scheduled_at: {:?})",
            result.id, args.side, args.quantity, args.symbol, args.price, args.leverage, args.order_type, args.condition, args.stop_loss_pct, args.stop_loss_price, args.scheduled_at
        ))]))
    }
}

impl TradingMcpServer {
    pub async fn call_list_symbols(&self) -> Result<CallToolResult, McpError> {
        self.list_symbols().await
    }

    pub async fn call_list_condition_variables(
        &self,
    ) -> Result<CallToolResult, McpError> {
        self.list_condition_variables().await
    }

    pub async fn call_create_trade(
        &self,
        args: Parameters<CreateTradeArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.create_trade(args).await
    }
}

// TODO tutaj trzeba wypełnić sekcję: Example conditions, te niżej przykładowe
#[tool_handler]
impl ServerHandler for TradingMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Trading MCP server for creating conditional trade orders.\n\
                \n\
                Available tools:\n\
                - list_symbols: list all available trading symbols\n\
                - list_condition_variables: see what variables are available for Lua conditions\n\
                - create_trade: create a trade order with a Lua condition expression\n\
                \n\
                The 'condition' field in create_trade is a Lua expression evaluated against live market data. \
                It must return a boolean. Use list_condition_variables to discover available variables.\n\
                \n\
                Example conditions:\n\
                - price < 129\n\
                - price < 130 and rsi < 30\n\
                - price < sma * 0.98 or rsi < 25\n\
                - macd > macd_signal and volume > 1000000"
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
