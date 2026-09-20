use mediadive_api::{AppState, config::Config, routes, telemetry};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    telemetry::init_tracing();
    let metrics = telemetry::init_metrics()?;
    let config = Config::from_env()?;

    let db = mediadive_db::connect(&config.database_url, config.db_max_connections).await?;
    let app = routes::router(AppState { db, metrics }).layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "api listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

/// Kubernetes sends SIGTERM before removing a pod; draining in-flight requests
/// is what makes a blue-green cutover invisible to callers.
async fn shutdown() {
    let ctrl_c = async { tokio::signal::ctrl_c().await.ok() };
    #[cfg(unix)]
    let term = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<Option<()>>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = term => {},
    }
    tracing::info!("shutdown signal received");
}
