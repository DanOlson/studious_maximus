#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = app::AppReadWrite::from_env().await?;
    app.update_courses().await?;

    Ok(())
}
