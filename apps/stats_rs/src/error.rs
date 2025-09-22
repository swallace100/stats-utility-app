use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("empty dataset")]
    Empty,
    #[error("not-a-number encountered")]
    NaN,
    #[error("failed to parse CSV")]
    CsvParse,
    #[error("no numeric data found in CSV")]
    NoNumeric,
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> axum::response::Response {
        // For now, all are 400; easy to specialize later.
        let status = match self {
            ServiceError::Empty
            | ServiceError::NaN
            | ServiceError::CsvParse
            | ServiceError::NoNumeric => StatusCode::BAD_REQUEST,
        };

        let body = json!({
            "error": self.to_string()
        });

        (status, Json(body)).into_response()
    }
}
