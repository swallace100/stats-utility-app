//! JSON Schema & OpenAPI exposure.

use axum::Json;
use axum::response::IntoResponse;
use schemars::schema_for;
use serde_json::json;

/// Return JSON Schema for `DescribeInput`.
pub async fn schema_describe_input() -> impl IntoResponse {
    Json(schema_for!(crate::types::DescribeInput))
}

/// Return JSON Schema for `DescribeOutput`.
pub async fn schema_describe_output() -> impl IntoResponse {
    Json(schema_for!(crate::types::DescribeOutput))
}

/// Minimal OpenAPI 3.0 document generated from `schemars` schemas.
///
/// Exposes the service surface used by Swagger/ReDoc UIs.
/// The document includes paths, summaries, and request/response schemas.
///
/// This is a **lightweight** OpenAPI; for production you may want a fuller
/// doc (e.g., with examples, tags, error schemas, etc.).
pub async fn openapi() -> impl IntoResponse {
    // ---- Schemas from your crate::types ----
    let s_describe_in = schema_for!(crate::types::DescribeInput);
    let s_describe_out = schema_for!(crate::types::DescribeOutput);
    let s_summary_in = schema_for!(crate::types::SummaryIn);
    let s_summary_out = schema_for!(crate::types::SummaryOut);
    let s_dist_in = schema_for!(crate::types::DistIn);
    let s_dist_out = schema_for!(crate::types::DistOut);
    let s_pair_in = schema_for!(crate::types::PairIn);
    let s_pair_out = schema_for!(crate::types::PairOut);
    let s_ecdf_in = schema_for!(crate::types::EcdfIn);
    let s_ecdf_out = schema_for!(crate::types::EcdfOut);
    let s_qq_in = schema_for!(crate::types::QqIn);
    let s_qq_out = schema_for!(crate::types::QqOut);
    let s_corr_in = schema_for!(crate::types::CorrMatrixIn);
    let s_corr_out = schema_for!(crate::types::CorrMatrixOut);
    let s_outliers_in = schema_for!(crate::types::OutliersIn);
    let s_outliers_out = schema_for!(crate::types::OutliersOut);
    let s_norm_in = schema_for!(crate::types::NormalizeIn);
    let s_norm_out = schema_for!(crate::types::NormalizeOut);
    let s_binrule_in = schema_for!(crate::types::BinRuleIn);
    let s_binrule_out = schema_for!(crate::types::BinRuleOut);

    Json(json!({
      "openapi": "3.0.3",
      "info": { "title": "stats_rs", "version": env!("CARGO_PKG_VERSION") },
      "paths": {
        // --- health ---
        "/api/v1/health": { "get": { "summary": "Liveness probe",  "responses": { "200": { "description": "OK" }}} },
        "/api/v1/ready":  { "get": { "summary": "Readiness probe", "responses": { "200": { "description": "OK" }}} },

        // --- describe ---
        "/api/v1/describe": {
          "post": {
            "summary": "Compute stats for JSON array of numbers",
            "requestBody": {"required": true, "content": {"application/json": {"schema": s_describe_in}}},
            "responses":   {"200": {"description": "OK", "content": {"application/json": {"schema": s_describe_out}}}, "400": {"description": "Bad Request"}}
          }
        },

        // --- describe CSV ---
        "/api/v1/describe-csv": {
          "post": {
            "summary": "Compute stats for CSV body (text/csv)",
            "requestBody": {"required": true, "content": {"text/csv": {"schema": {"type": "string", "format": "binary"}}}},
            "responses":   {"200": {"description": "OK", "content": {"application/json": {"schema": s_describe_out}}}, "400": {"description": "Bad Request"}}
          }
        },

        // --- summary ---
        "/api/v1/stats/summary": {
          "post": {"summary": "Summary statistics",
            "requestBody": {"required": true, "content": {"application/json": {"schema": s_summary_in}}},
            "responses":   {"200": {"description": "OK", "content": {"application/json": {"schema": s_summary_out}}}}
          }
        },

        // --- distribution ---
        "/api/v1/stats/distribution": {
          "post": {"summary": "Histogram, quantiles, skew/kurtosis, entropy",
            "requestBody": {"required": true, "content": {"application/json": {"schema": s_dist_in}}},
            "responses":   {"200": {"description": "OK", "content": {"application/json": {"schema": s_dist_out}}}}
          }
        },

        // --- pairwise x/y ---
        "/api/v1/stats/pairwise": {
          "post": {"summary": "Covariance and rank/linear correlations for two vectors",
            "requestBody": {"required": true, "content": {"application/json": {"schema": s_pair_in}}},
            "responses":   {"200": {"description": "OK", "content": {"application/json": {"schema": s_pair_out}}}}
          }
        },

        // --- ECDF ---
        "/api/v1/stats/ecdf": {
          "post": {"summary": "Empirical CDF (optionally downsampled)",
            "requestBody": {"required": true, "content": {"application/json": {"schema": s_ecdf_in}}},
            "responses":   {"200": {"description": "OK", "content": {"application/json": {"schema": s_ecdf_out}}}}
          }
        },

        // --- QQ Normal ---
        "/api/v1/stats/qq-normal": {
          "post": {"summary": "QQ-plot data against Normal reference (with μ, σ estimates)",
            "requestBody": {"required": true, "content": {"application/json": {"schema": s_qq_in}}},
            "responses":   {"200": {"description": "OK", "content": {"application/json": {"schema": s_qq_out}}}}
          }
        },

        // --- Correlation matrix ---
        "/api/v1/stats/corr-matrix": {
          "post": {"summary": "Correlation matrix for multiple series",
            "requestBody": {"required": true, "content": {"application/json": {"schema": s_corr_in}}},
            "responses":   {"200": {"description": "OK", "content": {"application/json": {"schema": s_corr_out}}}}
          }
        },

        // --- Outliers ---
        "/api/v1/stats/outliers": {
          "post": {"summary": "Outlier detection (IQR, z-score, etc.)",
            "requestBody": {"required": true, "content": {"application/json": {"schema": s_outliers_in}}},
            "responses":   {"200": {"description": "OK", "content": {"application/json": {"schema": s_outliers_out}}}}
          }
        },

        // --- Normalize ---
        "/api/v1/stats/normalize": {
          "post": {"summary": "Normalize vector (z-score or min–max range)",
            "requestBody": {"required": true, "content": {"application/json": {"schema": s_norm_in}}},
            "responses":   {"200": {"description": "OK", "content": {"application/json": {"schema": s_norm_out}}}}
          }
        },

        // --- Bin rule ---
        "/api/v1/stats/binrule": {
          "post": {"summary": "Pick number of histogram bins via rule",
            "requestBody": {"required": true, "content": {"application/json": {"schema": s_binrule_in}}},
            "responses":   {"200": {"description": "OK", "content": {"application/json": {"schema": s_binrule_out}}}}
          }
        }
      }
    }))
}
