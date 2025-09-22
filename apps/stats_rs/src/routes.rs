use std::sync::Arc;

use axum::body::Bytes;
use axum::response::Html;
use axum::{Json, extract::State, response::IntoResponse};
use schemars::schema_for;
use serde_json::json;

use crate::{
    error::ServiceError,
    state::AppState,
    stats::{mean, median, sample_std_dev},
    types::{DescribeInput, DescribeOutput},
};

pub async fn health() -> &'static str {
    "ok"
}

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

/// Accept raw CSV (text/csv). Collects all numeric fields from all rows/cols.
/// Returns 400 if CSV can't be parsed or no numeric cells are found.
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

fn parse_csv_numbers(bytes: &Bytes) -> Result<Vec<f64>, csv::Error> {
    // Try with headers=true first, then fallback to headers=false
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

pub async fn schema_describe_input() -> impl IntoResponse {
    Json(schema_for!(DescribeInput))
}

pub async fn schema_describe_output() -> impl IntoResponse {
    Json(schema_for!(DescribeOutput))
}

/// Minimal OpenAPI 3 document built from our schemars types.
pub async fn openapi() -> impl IntoResponse {
    let input_schema = schema_for!(crate::types::DescribeInput); // Schema
    let output_schema = schema_for!(crate::types::DescribeOutput); // Schema

    Json(json!({
        "openapi": "3.0.3",
        "info": { "title": "stats_rs", "version": "0.1.0" },
        "paths": {
            "/health": {
                "get": {
                    "summary": "Health check",
                    "responses": { "200": { "description": "OK" } }
                }
            },
            "/describe": {
                "post": {
                    "summary": "Compute stats for JSON array of numbers",
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": input_schema } }
                    },
                    "responses": {
                        "200": { "description": "OK",
                            "content": { "application/json": { "schema": output_schema } } },
                        "400": { "description": "Bad Request" }
                    }
                }
            },
            "/describe-csv": {
                "post": {
                    "summary": "Compute stats for CSV body (text/csv)",
                    "requestBody": {
                        "required": true,
                        "content": { "text/csv": { "schema": { "type": "string", "format": "binary" } } }
                    },
                    "responses": {
                        "200": { "description": "OK",
                            "content": { "application/json": { "schema": output_schema } } },
                        "400": { "description": "Bad Request" }
                    }
                }
            }
        }
    }))
}

pub async fn swagger_ui() -> Html<&'static str> {
    // Minimal Swagger UI via CDN; it loads your existing /openapi.json
    Html(
        r#"
<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>stats_rs – API Docs</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="stylesheet"
      href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
  </head>
  <body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>
      window.ui = SwaggerUIBundle({
        url: '/openapi.json',
        dom_id: '#swagger-ui',
        presets: [SwaggerUIBundle.presets.apis],
        layout: 'BaseLayout'
      });
    </script>
  </body>
</html>
"#,
    )
}
