//! HTTP surface. Routing, middleware, handlers.

pub mod config;
pub mod observe;
pub mod routes;
pub mod telemetry;

use mediadive_db::Db;
use metrics_exporter_prometheus::PrometheusHandle;
use utoipa::OpenApi;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub metrics: PrometheusHandle,
}

/// The generated contract. `cargo run --bin openapi` emits it; CI fails if the
/// committed spec differs. See `docs/adr/0007-generated-contracts.md`.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "mediadive",
        description = "Media tracking and discovery. Terms in docs/CONTEXT.md."
    ),
    paths(routes::version),
    components(schemas(mediadive_contracts::Version, mediadive_contracts::Problem))
)]
pub struct ApiDoc;
