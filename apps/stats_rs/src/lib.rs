// src/lib.rs
pub mod error;
pub mod routes;
pub mod state;
pub mod stats;
pub mod types;

use axum::{
    Router,
    routing::{get, post},
};
use state::AppState;
use std::sync::Arc;

pub fn build_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(routes::health))
        .route("/describe", post(routes::describe))
        .route("/describe-csv", post(routes::describe_csv))
        .route("/schema/describe-input", get(routes::schema_describe_input))
        .route(
            "/schema/describe-output",
            get(routes::schema_describe_output),
        )
        .route("/openapi.json", get(routes::openapi))
        .with_state(state)
}
