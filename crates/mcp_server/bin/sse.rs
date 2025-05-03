use std::{sync::Arc, time::Duration};

use app::AppReadonly;
use mcp_server::{Result, School};
use rmcp::transport::sse_server::SseServer;
use tracing_subscriber::EnvFilter;

const BIND_ADDRESS: &str = "127.0.0.1:3030";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting MCP Server");

    let app = Arc::new(AppReadonly::from_env().await?);

    let ct = SseServer::serve(BIND_ADDRESS.parse()?)
        .await?
        .with_service(move || School::new(app.clone()));

    tokio::signal::ctrl_c().await?;
    tracing::info!("Received Ctrl+C. Shutting down server...");
    ct.cancel();

    tokio::time::sleep(Duration::from_millis(1_000)).await;
    tracing::info!("Exiting");

    Ok(())
}
