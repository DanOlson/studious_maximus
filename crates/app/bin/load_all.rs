#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = app::AppReadWrite::from_env().await?;
    app.sync_all().await?;

    Ok(())
}
