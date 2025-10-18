//! # Shared Application State
//!
//! This module defines the [`AppState`] struct — the central shared state
//! passed to all Axum route handlers in the `stats_rs` microservice.
//!
//! ## Overview
//!
//! The state is wrapped in an [`Arc`](std::sync::Arc) and cloned into
//! each request handler via Axum’s `.with_state()` mechanism.
//!
//! While currently empty, this struct serves as a foundation for adding
//! shared resources such as:
//!
//! - Database connection pools (e.g., PostgreSQL, MongoDB, SQLite)
//! - Cached models or compiled statistical kernels
//! - Configuration data or feature flags
//! - Global rate limiter or metrics handles
//!
//! Example usage from [`lib.rs`](crate::build_app):
//!
//! ```rust,ignore
//! let state = Arc::new(AppState::default());
//! let app = build_app(state);
//! ```
//!
//! Each handler that needs shared context can extract it with:
//!
//! ```rust,ignore
//! async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
//!     // access shared resources here
//! }
//! ```

/// Global shared state for the `stats_rs` service.
///
/// Cloned and shared across all request handlers.
/// Implements [`Clone`] and [`Default`] for convenience in both testing
/// and production.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Clone, Default)]
/// pub struct AppState {
///     pub db: Pool<Postgres>,
///     pub cache: Arc<Mutex<HashMap<String, f64>>>,
/// }
/// ```
///
/// The current implementation is an empty struct, ready for extension
/// as the microservice evolves.
#[derive(Clone, Default)]
pub struct AppState;
