//! Prometheus exposition (stub).

/// Minimal Prometheus exposition format stub.
///
/// Replace with real metrics if enabling the `"metrics"` feature.
pub async fn prom_metrics() -> &'static str {
    "# HELP dummy 1\n# TYPE dummy counter\ndummy 1\n"
}
