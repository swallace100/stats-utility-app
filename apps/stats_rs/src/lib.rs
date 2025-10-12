// src/lib.rs
pub mod error;
pub mod routes;
pub mod state;
pub mod stats;
pub mod types;

use axum::extract::DefaultBodyLimit;
use axum::{
    Router,
    http, // <-- for http::Method
    routing::{get, post},
};
use state::AppState;
use std::{sync::Arc, time::Duration};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

pub fn build_app(state: Arc<AppState>) -> Router {
    // --- v1 API ---
    let v1 = Router::new()
        // health/readiness
        .route("/healthz", get(routes::health))
        .route("/readyz", get(routes::ready))
        // existing describe endpoints
        .route("/describe", post(routes::describe))
        .route("/describe-csv", post(routes::describe_csv))
        .route("/schema/describe-input", get(routes::schema_describe_input))
        .route(
            "/schema/describe-output",
            get(routes::schema_describe_output),
        )
        // core stats endpoints
        .route("/stats/summary", post(routes::stats_summary))
        .route("/stats/distribution", post(routes::stats_distribution))
        .route("/stats/pairwise", post(routes::stats_pairwise))
        .with_state(state.clone());

    // RAG metrics are optional (behind the "rag" feature)
    #[cfg(feature = "rag")]
    {
        v1 = v1.route("/stats/rag/metrics", post(routes::stats_rag_metrics));
    }

    // --- root router ---
    let root = Router::new()
        .nest("/api/v1", v1)
        // always expose OpenAPI JSON if you have it
        .route("/openapi.json", get(routes::openapi))
        // middleware layers
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(
            CorsLayer::new()
                .allow_methods([http::Method::GET, http::Method::POST, http::Method::OPTIONS])
                .allow_origin(Any)
                .allow_headers(Any),
        )
        .layer(DefaultBodyLimit::max(25 * 1024 * 1024)) // allow larger CSV uploads
        .layer(TimeoutLayer::new(Duration::from_secs(30)));

    // Optional docs UI (e.g., ReDoc/Swagger) behind "docs" feature
    #[cfg(feature = "docs")]
    {
        root = root.route("/docs", get(routes::docs_ui));
    }

    // Optional Prometheus metrics behind "metrics" feature
    #[cfg(feature = "metrics")]
    {
        root = root.route("/metrics", get(routes::prom_metrics));
    }

    root
}
