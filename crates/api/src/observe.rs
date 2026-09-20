//! Request metrics.
//!
//! The Prometheus exporter renders an empty body until something records a
//! measurement, so these exist partly to make `/metrics` provably alive.

use std::time::Instant;

use axum::{extract::MatchedPath, extract::Request, middleware::Next, response::Response};

pub async fn track(request: Request, next: Next) -> Response {
    // The matched route pattern, not the concrete path — otherwise every id
    // becomes its own label value and the series count grows without bound.
    let path = request
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_owned())
        .unwrap_or_else(|| "unmatched".to_owned());
    let method = request.method().to_string();

    let started = Instant::now();
    let response = next.run(request).await;
    let status = response.status().as_u16().to_string();

    metrics::counter!(
        "http_requests_total",
        "method" => method.clone(),
        "path" => path.clone(),
        "status" => status,
    )
    .increment(1);
    metrics::histogram!(
        "http_request_duration_seconds",
        "method" => method,
        "path" => path,
    )
    .record(started.elapsed().as_secs_f64());

    response
}
