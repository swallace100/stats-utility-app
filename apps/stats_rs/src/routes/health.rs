// ---------------- Health / Ready ----------------

use crate::state::AppState;
use axum::extract::State;
use std::sync::Arc;

/// Liveness probe.
///
/// Returns a static `"ok"` string. Useful for container health checks.
pub async fn health() -> &'static str {
    "ok"
}

/// Readiness probe.
///
/// Returns `"ready"` once the service is able to handle requests.
/// In the future, this may check shared resources in [`AppState`].
pub async fn ready(State(_state): State<Arc<AppState>>) -> &'static str {
    "ready"
}
