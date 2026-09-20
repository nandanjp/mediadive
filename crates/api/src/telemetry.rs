//! Structured logs and a Prometheus registry.

use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use tracing_subscriber::EnvFilter;

/// JSON to stdout, so Loki needs no parsing rules.
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}

pub fn init_metrics() -> anyhow::Result<PrometheusHandle> {
    PrometheusBuilder::new()
        .install_recorder()
        .map_err(|e| anyhow::anyhow!("installing prometheus recorder: {e}"))
}
