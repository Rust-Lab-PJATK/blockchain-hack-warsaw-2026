use async_trait::async_trait;
use axum::{Extension, Router as AxumRouter};
use loco_rs::{
    app::{AppContext, Hooks, Initializer},
    bgworker::Queue,
    boot::{create_app, BootResult, StartMode},
    config::Config,
    controller::AppRoutes,
    environment::Environment,
    task::Tasks,
    Result,
};
use migration::Migrator;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpService,
};
use std::path::Path;
use std::sync::Arc;

#[allow(unused_imports)]
use crate::{controllers, services, tasks};

pub struct App;
#[async_trait]
impl Hooks for App {
    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: Config,
    ) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment, config).await
    }

    async fn after_routes(router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
        let mcp_server = Arc::new(services::mcp::TradingMcpServer::new(ctx.db.clone()));

        let provider: Arc<dyn services::llm::LlmProvider> =
            match services::llm::VercelProvider::new(mcp_server.clone()) {
                Ok(p) => {
                    tracing::info!("Using Vercel AI Gateway provider");
                    Arc::new(p)
                }
                Err(e) => {
                    tracing::warn!("Vercel provider not configured ({e}), using MockProvider");
                    Arc::new(services::llm::MockProvider)
                }
            };

        let db = ctx.db.clone();
        let mcp_service = StreamableHttpService::new(
            move || Ok(services::mcp::TradingMcpServer::new(db.clone())),
            LocalSessionManager::default().into(),
            Default::default(),
        );

        // Start the strategy engine background loop
        let drift_provider: Arc<dyn services::drift::DriftProvider> =
            Arc::from(services::drift::create_drift_provider(ctx).await?);
        services::strategy_engine::start(ctx.db.clone(), drift_provider);

        Ok(router
            .layer(Extension(provider))
            .nest_service("/mcp", mcp_service))
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![])
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes() // controller routes below
            .add_route(controllers::strategies::routes())
            .add_route(controllers::chat::routes())
    }
    async fn connect_workers(_ctx: &AppContext, _queue: &Queue) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn register_tasks(tasks: &mut Tasks) {
        // tasks-inject (do not remove)
    }
    async fn truncate(_ctx: &AppContext) -> Result<()> {
        Ok(())
    }
    async fn seed(_ctx: &AppContext, _base: &Path) -> Result<()> {
        Ok(())
    }
}