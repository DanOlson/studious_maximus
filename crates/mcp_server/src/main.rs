mod result;
mod school;

use app::App;
pub use result::Result;
use rmcp::{ServiceExt, transport::stdio};
use school::School;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting MCP Server");

    let app = App::from_env().await?;

    let service = School::new(app).serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {}", e);
    })?;

    service.waiting().await?;
    Ok(())
}
