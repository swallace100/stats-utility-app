// ---------------- Describe (JSON & CSV) ----------------

use crate::{
    error::ServiceError,
    state::AppState,
    stats::prelude::*,
    types::{DescribeInput, DescribeOutput},
};
use axum::{Json, body::Bytes, extract::State};
use std::sync::Arc;

/// Compute simple descriptive stats for a JSON array of numbers.
///
/// Validates input for emptiness and `NaN`/non-finite values.
/// Returns `400 Bad Request` via [`ServiceError`] on invalid input.
///
/// - **Request**: [`DescribeInput`] (`application/json`)
/// - **Response**: [`DescribeOutput`] (`200 OK`) or error (`400`)
pub async fn describe(
    State(_state): State<Arc<AppState>>,
    Json(input): Json<DescribeInput>,
) -> Result<Json<DescribeOutput>, ServiceError> {
    let nums = input.0;
    if nums.is_empty() {
        return Err(ServiceError::Empty);
    }
    if nums.iter().any(|v| v.is_nan() || !v.is_finite()) {
        return Err(ServiceError::NaN);
    }

    let count = nums.len();
    let mean = mean(&nums);
    let median = median(&nums);
    let std_dev = sample_std_dev(&nums, mean);
    Ok(Json(DescribeOutput {
        count,
        mean,
        median,
        std_dev,
    }))
}

/// Compute descriptive stats from a raw CSV payload (`text/csv`).
///
/// The parser scans all fields in all rows, collecting cells that parse as `f64`.
/// Tries first with `has_headers=true`, then falls back to `false`.
///
/// - **Request**: body `text/csv`
/// - **Response**: [`DescribeOutput`] (`200 OK`)
/// - **Errors**: `CsvParse` (malformed CSV), `NoNumeric` (no numeric cells)
pub async fn describe_csv(
    State(_state): State<Arc<AppState>>,
    body: Bytes,
) -> Result<Json<DescribeOutput>, ServiceError> {
    let nums = parse_csv_numbers(&body).map_err(|_| ServiceError::CsvParse)?;
    if nums.is_empty() {
        return Err(ServiceError::NoNumeric);
    }

    let count = nums.len();
    let mean = mean(&nums);
    let median = median(&nums);
    let std_dev = sample_std_dev(&nums, mean);
    Ok(Json(DescribeOutput {
        count,
        mean,
        median,
        std_dev,
    }))
}

/// Parse all numeric cells from a CSV byte buffer.
fn parse_csv_numbers(bytes: &Bytes) -> Result<Vec<f64>, csv::Error> {
    let try_once = |has_headers: bool| -> Result<Vec<f64>, csv::Error> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(has_headers)
            .flexible(true)
            .from_reader(bytes.as_ref());
        let mut v = Vec::new();
        for result in rdr.records() {
            let rec = result?;
            for field in rec.iter() {
                if let Ok(x) = field.trim().parse::<f64>() {
                    v.push(x);
                }
            }
        }
        Ok(v)
    };

    let mut out = try_once(true)?;
    if out.is_empty() {
        out = try_once(false)?;
    }
    Ok(out)
}
