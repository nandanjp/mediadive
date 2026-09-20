//! Handlers.
//!
//! Conventions — `/api/v1` prefix, snake_case JSON, RFC 9457 errors with a
//! stable `code` — are in `docs/API.md`.

use axum::{Json, Router, extract::State, http::StatusCode, middleware, routing::get};
use mediadive_contracts::Version;
use utoipa::OpenApi;

use crate::{ApiDoc, AppState};

/// Git SHA, injected at image build time.
const REVISION: &str = match option_env!("MEDIADIVE_GIT_SHA") {
    Some(sha) => sha,
    None => "dev",
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/metrics", get(metrics))
        .route("/api/v1/version", get(version))
        .route("/api/v1/openapi.json", get(openapi))
        .layer(middleware::from_fn(crate::observe::track))
        .with_state(state)
}

/// Liveness. The process is up; no dependency is consulted, so a database blip
/// never triggers a pod restart.
async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

/// Readiness. Dependencies are reachable, so an unready pod leaves the load
/// balancer instead of serving errors.
async fn ready(State(state): State<AppState>) -> StatusCode {
    match mediadive_db::ping(&state.db).await {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(error) => {
            tracing::warn!(%error, "readiness check failed");
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}

async fn metrics(State(state): State<AppState>) -> String {
    state.metrics.render()
}

/// Build identity. The smoke suite asserts this matches the SHA CI built.
#[utoipa::path(
    get,
    path = "/api/v1/version",
    responses((status = 200, description = "Build identity", body = Version))
)]
pub async fn version() -> Json<Version> {
    Json(Version {
        revision: REVISION.to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
    })
}

async fn openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
