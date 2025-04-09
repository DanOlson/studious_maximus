#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = app::App::from_env().await?;
    let _ = app.update_students().await;

    Ok(())
}
