//! # `stats_rs` Microservice Library
//!
//! The `stats_rs` crate provides the core HTTP service for statistical computation
//! within the **Stats Utility App**. It exposes a RESTful API built with Axum,
//! performing numerical analysis, normalization, and summary operations on uploaded data.
//!
//! ## Overview
//!
//! The library exports modular components organized as follows:
//!
//! - [`error`] — Standardized error types for API and computation failures.
//! - [`routes`] — HTTP route handlers for each statistical endpoint.
//! - [`state`] — Global [`AppState`] shared across handlers.
//! - [`stats`] — Core statistical algorithms (mean, variance, correlation, etc.).
//! - [`types`] — Shared request/response DTOs and Zod-compatible schemas.
//!
//! The central entry point is [`build_app`], which assembles the Axum router
//! with all endpoints, middleware, and feature-conditional routes.

pub mod error;
pub mod routes;
pub mod state;
pub mod stats;
pub mod types;

use axum::extract::DefaultBodyLimit;
use axum::{
    Router, http,
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

/// Builds and configures the top-level Axum [`Router`] for the `stats_rs` microservice.
///
/// This function wires up all routes, middleware layers, and optional feature-based
/// extensions (e.g., `/metrics`, `/docs`, or `/stats/rag/metrics`). It is the canonical
/// entry point used by `main.rs` or containerized deployments.
///
/// # Parameters
///
/// - `state`: Shared [`AppState`] reference wrapped in [`Arc`].
///   Contains service-wide configuration, database handles, or cached models.
///
/// # Routes
///
/// The router exposes a versioned API under `/api/v1`, including:
///
/// | Category | Path | Method | Description |
/// |-----------|------|---------|-------------|
/// | Health    | `/health`, `/ready` | `GET` | Liveness and readiness checks |
/// | Describe  | `/describe`, `/describe-csv` | `POST` | Statistical summaries for JSON or CSV input |
/// | Schemas   | `/schema/*` | `GET` | Returns JSON schemas for input/output payloads |
/// | Core Stats | `/stats/summary`, `/stats/distribution`, `/stats/pairwise` | `POST` | Core analytic endpoints |
/// | Extended Stats | `/stats/ecdf`, `/stats/qq-normal`, `/stats/corr-matrix`, `/stats/outliers`, `/stats/normalize`, `/stats/binrule` | `POST` | Advanced statistical and normalization routines |
///
/// Feature-based optional routes:
///
/// - `rag` → `/stats/rag/metrics` for retrieval-augmented generation metrics
/// - `docs` → `/docs` for Swagger/ReDoc UI
/// - `metrics` → `/metrics` for Prometheus scraping
///
/// # Middleware
///
/// The following layers are attached to the root router:
///
/// - [`TraceLayer`] for structured HTTP logging
/// - [`CompressionLayer`] for gzip/br encoding
/// - [`CorsLayer`] permitting any origin and standard methods
/// - [`DefaultBodyLimit`] increased to 25 MB (large CSVs)
/// - [`TimeoutLayer`] limiting request duration to 30 s
///
/// # Example
///
/// ```rust,ignore
/// let app = build_app(Arc::new(AppState::default()));
/// tracing::info!("Server ready on :9000");
/// axum::serve(listener, app).await?;
/// ```
///
/// # Returns
///
/// An Axum [`Router`] instance ready to be served by a Tokio runtime.
pub fn build_app(state: Arc<AppState>) -> Router {
    // --- v1 API ---
    let v1 = Router::new()
        // Health and readiness endpoints
        .route("/health", get(routes::health))
        .route("/ready", get(routes::ready))
        // "Describe" endpoints: summarize numeric arrays or CSV files
        .route("/describe", post(routes::describe))
        .route("/describe-csv", post(routes::describe_csv))
        // JSON schema reflection for input/output
        .route("/schema/describe-input", get(routes::schema_describe_input))
        .route(
            "/schema/describe-output",
            get(routes::schema_describe_output),
        )
        // Core statistics endpoints
        .route("/stats/summary", post(routes::stats_summary))
        .route("/stats/distribution", post(routes::stats_distribution))
        .route("/stats/pairwise", post(routes::stats_pairwise))
        // Extended statistics
        .route("/stats/ecdf", post(routes::stats_ecdf))
        .route("/stats/qq-normal", post(routes::stats_qq_normal))
        .route("/stats/corr-matrix", post(routes::stats_corr_matrix))
        .route("/stats/outliers", post(routes::stats_outliers))
        .route("/stats/normalize", post(routes::stats_normalize))
        .route("/stats/binrule", post(routes::stats_binrule))
        .with_state(state.clone());

    // Feature: retrieval-augmented metrics (RAG)
    #[cfg(feature = "rag")]
    let v1 = v1.route("/stats/rag/metrics", post(routes::stats_rag_metrics));

    // --- root router ---
    let root = Router::new()
        .nest("/api/v1", v1)
        // Always expose raw OpenAPI JSON (generated by backend or contracts)
        .route("/openapi.json", get(routes::openapi))
        // Middleware layers
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(
            CorsLayer::new()
                .allow_methods([http::Method::GET, http::Method::POST, http::Method::OPTIONS])
                .allow_origin(Any)
                .allow_headers(Any),
        )
        .layer(DefaultBodyLimit::max(25 * 1024 * 1024)) // allow large CSV uploads
        .layer(TimeoutLayer::new(Duration::from_secs(30)));

    // Feature: documentation UI
    #[cfg(feature = "docs")]
    {
        root = root.route("/docs", get(routes::docs_ui));
    }

    // Feature: Prometheus metrics
    #[cfg(feature = "metrics")]
    {
        root = root.route("/metrics", get(routes::prom_metrics));
    }

    root
}
