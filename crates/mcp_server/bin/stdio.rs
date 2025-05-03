use std::sync::Arc;

use app::AppReadonly;
use mcp_server::Result;
use mcp_server::School;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting MCP Server");

    let app = Arc::new(AppReadonly::from_env().await?);

    let service = School::new(app).serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {}", e);
    })?;

    service.waiting().await?;
    Ok(())
}
