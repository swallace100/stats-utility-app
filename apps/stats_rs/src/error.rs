//! # Error Handling Module for `stats_rs`
//!
//! This module defines the core [`ServiceError`] enum used across the `stats_rs`
//! microservice to represent high-level service and data processing errors.
//!
//! Each variant corresponds to a common failure mode during CSV ingestion,
//! numeric analysis, or data validation. The enum also implements [`IntoResponse`]
//! so it can be returned directly from Axum handlers as structured JSON
//! error responses.

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

/// Represents errors that may occur while processing statistical requests.
///
/// The [`ServiceError`] type captures the most common runtime failures encountered
/// in the `stats_rs` microservice, including malformed or missing data.
/// Each variant includes a descriptive `Display` message (via `thiserror`)
/// for both developer debugging and user-facing API responses.
///
/// # Example
///
/// ```rust,ignore
/// use crate::error::ServiceError;
///
/// fn compute_mean(xs: &[f64]) -> Result<f64, ServiceError> {
///     if xs.is_empty() {
///         return Err(ServiceError::Empty);
///     }
///     if xs.iter().any(|x| x.is_nan()) {
///         return Err(ServiceError::NaN);
///     }
///     Ok(xs.iter().sum::<f64>() / xs.len() as f64)
/// }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// The input dataset is empty or contains no usable values.
    ///
    /// Typically returned when the user uploads a CSV or JSON array
    /// with zero rows or only missing values.
    #[error("empty dataset")]
    Empty,

    /// The dataset contained at least one `NaN` (not-a-number) value
    /// where numeric input was expected.
    ///
    /// This variant helps prevent invalid floating-point operations.
    #[error("not-a-number encountered")]
    NaN,

    /// The uploaded CSV could not be parsed into valid records.
    ///
    /// This often indicates malformed delimiters, inconsistent headers,
    /// or corrupted encodings. In the future, this error may include
    /// more detailed location context (e.g., row and column).
    #[error("failed to parse CSV")]
    CsvParse,

    /// The CSV was parsed successfully but contained no numeric columns.
    ///
    /// Since most analyses require at least one numeric column,
    /// this is returned when all fields are strings, booleans, or empty.
    #[error("no numeric data found in CSV")]
    NoNumeric,
}

impl IntoResponse for ServiceError {
    /// Converts a [`ServiceError`] into an Axum `Response`.
    ///
    /// The implementation currently maps all variants to HTTP `400 Bad Request`,
    /// but can be extended in the future for more granular codes:
    ///
    /// | Variant | Status Code | Typical Meaning |
    /// |----------|--------------|----------------|
    /// | `Empty` | `400` | User provided an empty dataset |
    /// | `NaN` | `400` | Dataset contained invalid numeric values |
    /// | `CsvParse` | `400` | CSV could not be parsed |
    /// | `NoNumeric` | `400` | CSV contained no numeric data |
    ///
    /// The response body is JSON with a single `"error"` key, e.g.:
    ///
    /// ```json
    /// { "error": "empty dataset" }
    /// ```
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// async fn handler() -> Result<Json<Value>, ServiceError> {
    ///     Err(ServiceError::CsvParse)
    /// }
    /// ```
    fn into_response(self) -> axum::response::Response {
        // For now, all are mapped to HTTP 400; specialized status codes can be added later.
        let status = match self {
            ServiceError::Empty
            | ServiceError::NaN
            | ServiceError::CsvParse
            | ServiceError::NoNumeric => StatusCode::BAD_REQUEST,
        };

        let body = json!({ "error": self.to_string() });

        (status, Json(body)).into_response()
    }
}
