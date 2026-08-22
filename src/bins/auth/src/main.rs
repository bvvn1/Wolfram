use axum::Router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9999").await?;

    axum::serve(listener, app).await?;

    Ok(())
}
