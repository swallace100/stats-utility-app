use std::sync::Arc;

use axum::{
    Json,
    body::Bytes,
    extract::State,
    response::{Html, IntoResponse},
};
use schemars::schema_for;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    error::ServiceError,
    state::AppState,
    stats::prelude::*,
    types::{DescribeInput, DescribeOutput},
};

// ---------------- Health / Ready ----------------

pub async fn health() -> &'static str {
    "ok"
}

pub async fn ready(State(_state): State<Arc<AppState>>) -> &'static str {
    "ready"
}

// ---------------- Describe (JSON & CSV) ----------------

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

// ---------------- Schemas & OpenAPI ----------------

pub async fn schema_describe_input() -> impl IntoResponse {
    Json(schema_for!(DescribeInput))
}

pub async fn schema_describe_output() -> impl IntoResponse {
    Json(schema_for!(DescribeOutput))
}

/// Minimal OpenAPI 3 document built from our schemars types.
pub async fn openapi() -> impl IntoResponse {
    let input_schema = schema_for!(crate::types::DescribeInput);
    let output_schema = schema_for!(crate::types::DescribeOutput);

    Json(json!({
        "openapi": "3.0.3",
        "info": { "title": "stats_rs", "version": env!("CARGO_PKG_VERSION") },
        "paths": {
            "/api/v1/healthz": { "get": { "summary": "Health check", "responses": { "200": { "description": "OK" } } } },
            "/api/v1/readyz":  { "get": { "summary": "Readiness check", "responses": { "200": { "description": "OK" } } } },
            "/api/v1/describe": {
                "post": {
                    "summary": "Compute stats for JSON array of numbers",
                    "requestBody": { "required": true, "content": { "application/json": { "schema": input_schema } } },
                    "responses": {
                        "200": { "description": "OK", "content": { "application/json": { "schema": output_schema } } },
                        "400": { "description": "Bad Request" }
                    }
                }
            },
            "/api/v1/describe-csv": {
                "post": {
                    "summary": "Compute stats for CSV body (text/csv)",
                    "requestBody": { "required": true, "content": { "text/csv": { "schema": { "type": "string", "format": "binary" } } } },
                    "responses": {
                        "200": { "description": "OK", "content": { "application/json": { "schema": output_schema } } },
                        "400": { "description": "Bad Request" }
                    }
                }
            },
            "/api/v1/stats/summary":      { "post": { "summary": "Summary stats"      }},
            "/api/v1/stats/distribution": { "post": { "summary": "Histogram & shape"  }},
            "/api/v1/stats/pairwise":     { "post": { "summary": "Pairwise correlations" }}
        }
    }))
}

/// Minimal Swagger UI via CDN; it loads your existing /openapi.json
pub async fn swagger_ui() -> Html<&'static str> {
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

/// `lib.rs` expects `/docs`; reuse Swagger UI.
pub async fn docs_ui() -> Html<&'static str> {
    swagger_ui().await
}

// ---------------- Core Stats Endpoints ----------------

#[derive(Deserialize)]
pub struct ValuesIn {
    pub values: Vec<f64>,
}

#[derive(Serialize)]
pub struct SummaryOut {
    pub count: usize,
    pub mean: Option<f64>,
    pub median: Option<f64>,
    pub std: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub iqr: Option<f64>,
    pub mad: Option<f64>,
}

#[inline]
fn nan_to_opt(x: f64) -> Option<f64> {
    if x.is_nan() { None } else { Some(x) }
}

pub async fn stats_summary(Json(inp): Json<ValuesIn>) -> Json<SummaryOut> {
    let n = inp.values.len();
    if n == 0 {
        return Json(SummaryOut {
            count: 0,
            mean: None,
            median: None,
            std: None,
            min: None,
            max: None,
            iqr: None,
            mad: None,
        });
    }

    let m = mean(&inp.values);
    let med = median(&inp.values);
    let stdv = sample_std_dev(&inp.values, m);
    let mn = min(&inp.values);
    let mx = max(&inp.values);
    let i = iqr(&inp.values);
    let md = mad(&inp.values);

    Json(SummaryOut {
        count: n,
        mean: nan_to_opt(m),
        median: nan_to_opt(med),
        std: nan_to_opt(stdv),
        min: nan_to_opt(mn),
        max: nan_to_opt(mx),
        iqr: nan_to_opt(i),
        mad: nan_to_opt(md),
    })
}

#[derive(Deserialize)]
pub struct DistIn {
    pub values: Vec<f64>,
    pub bins: Option<usize>,
    pub quantiles: Option<Vec<f64>>,
}

#[derive(Serialize)]
pub struct DistOut {
    pub counts: Vec<usize>,
    pub edges: Vec<f64>,
    pub quantiles: Vec<(f64, f64)>, // (p, value)
    pub skewness: Option<f64>,
    pub excess_kurtosis: Option<f64>,
    pub entropy_bits: Option<f64>,
}

pub async fn stats_distribution(Json(inp): Json<DistIn>) -> Json<DistOut> {
    let values = inp.values;
    let n = values.len();
    if n == 0 {
        return Json(DistOut {
            counts: vec![],
            edges: vec![],
            quantiles: vec![],
            skewness: None,
            excess_kurtosis: None,
            entropy_bits: None,
        });
    }

    let bins = inp.bins.unwrap_or(10).max(2);
    let lo = min(&values);
    let hi = max(&values);
    let width = (hi - lo) / bins as f64;

    let mut counts = vec![0usize; bins];
    if width == 0.0 {
        counts[0] = n;
    } else {
        for &x in &values {
            let mut b = ((x - lo) / width).floor() as usize;
            if b >= bins {
                b = bins - 1;
            }
            counts[b] += 1;
        }
    }

    let mut edges = Vec::with_capacity(bins + 1);
    for i in 0..=bins {
        edges.push(lo + i as f64 * width);
    }

    let qs = inp.quantiles.unwrap_or_else(|| vec![0.25, 0.5, 0.75]);
    let quantiles: Vec<(f64, f64)> = qs.into_iter().map(|p| (p, quantile(&values, p))).collect();

    let sk = skewness(&values);
    let ek = excess_kurtosis(&values);

    let total = n as f64;
    let probs: Vec<f64> = counts.iter().map(|&c| c as f64 / total).collect();
    let h = entropy_bits(&probs);

    Json(DistOut {
        counts,
        edges,
        quantiles,
        skewness: nan_to_opt(sk),
        excess_kurtosis: nan_to_opt(ek),
        entropy_bits: nan_to_opt(h),
    })
}

#[derive(Deserialize)]
pub struct PairIn {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

#[derive(Serialize)]
pub struct PairOut {
    pub covariance: Option<f64>,
    pub pearson: Option<f64>,
    pub spearman: Option<f64>,
    pub kendall: Option<f64>,
}

pub async fn stats_pairwise(Json(inp): Json<PairIn>) -> Json<PairOut> {
    if inp.x.len() != inp.y.len() || inp.x.is_empty() {
        return Json(PairOut {
            covariance: None,
            pearson: None,
            spearman: None,
            kendall: None,
        });
    }
    let cov = covariance(&inp.x, &inp.y);
    let p = pearson_correlation(&inp.x, &inp.y);
    let s = spearman_rho(&inp.x, &inp.y);
    let k = kendall_tau_b(&inp.x, &inp.y);
    Json(PairOut {
        covariance: nan_to_opt(cov),
        pearson: nan_to_opt(p),
        spearman: nan_to_opt(s),
        kendall: nan_to_opt(k),
    })
}

// ---------------- Optional Prometheus stub ----------------

pub async fn prom_metrics() -> &'static str {
    "# HELP dummy 1\n# TYPE dummy counter\ndummy 1\n"
}
