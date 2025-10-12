use std::sync::Arc;

use axum::{
    Json,
    body::Bytes,
    extract::State,
    response::{Html, IntoResponse},
};
use schemars::schema_for;
use serde::Deserialize;
use serde_json::json;

use crate::{
    error::ServiceError,
    state::AppState,
    stats::prelude::*,
    types::{
        BinRuleIn, BinRuleOut, CorrMatrixIn, CorrMatrixOut, CorrMethod, DescribeInput,
        DescribeOutput, DistIn, DistOut, EcdfIn, EcdfOut, NormMethod, NormalizeIn, NormalizeOut,
        OutlierMethod, OutliersIn, OutliersOut, PairIn, PairOut, QqIn, QqOut, SummaryIn,
        SummaryOut,
    },
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

    axum::Json(json!({
      "openapi": "3.0.3",
      "info": { "title": "stats_rs", "version": env!("CARGO_PKG_VERSION") },
      "paths": {
        // --- health ---
        "/api/v1/health": { "get": { "summary": "Liveness probe",  "responses": { "200": { "description": "OK" } } } },
        "/api/v1/ready":  { "get": { "summary": "Readiness probe", "responses": { "200": { "description": "OK" } } } },

        // --- describe (JSON array) ---
        "/api/v1/describe": {
          "post": {
            "summary": "Compute stats for JSON array of numbers",
            "requestBody": {
              "required": true,
              "content": { "application/json": { "schema": s_describe_in } }
            },
            "responses": {
              "200": { "description": "OK", "content": { "application/json": { "schema": s_describe_out } } },
              "400": { "description": "Bad Request" }
            }
          }
        },

        // --- describe CSV ---
        "/api/v1/describe-csv": {
          "post": {
            "summary": "Compute stats for CSV body (text/csv)",
            "requestBody": {
              "required": true,
              "content": { "text/csv": { "schema": { "type": "string", "format": "binary" } } }
            },
            "responses": {
              "200": { "description": "OK", "content": { "application/json": { "schema": s_describe_out } } },
              "400": { "description": "Bad Request" }
            }
          }
        },

        // --- summary ---
        "/api/v1/stats/summary": {
          "post": {
            "summary": "Summary statistics",
            "requestBody": { "required": true, "content": { "application/json": { "schema": s_summary_in } } },
            "responses": { "200": { "description": "OK", "content": { "application/json": { "schema": s_summary_out } } } }
          }
        },

        // --- distribution ---
        "/api/v1/stats/distribution": {
          "post": {
            "summary": "Histogram, quantiles, skew/kurtosis, entropy",
            "requestBody": { "required": true, "content": { "application/json": { "schema": s_dist_in } } },
            "responses": { "200": { "description": "OK", "content": { "application/json": { "schema": s_dist_out } } } }
          }
        },

        // --- pairwise x/y ---
        "/api/v1/stats/pairwise": {
          "post": {
            "summary": "Covariance and rank/linear correlations for two vectors",
            "requestBody": { "required": true, "content": { "application/json": { "schema": s_pair_in } } },
            "responses": { "200": { "description": "OK", "content": { "application/json": { "schema": s_pair_out } } } }
          }
        },

        // --- ECDF ---
        "/api/v1/stats/ecdf": {
          "post": {
            "summary": "Empirical CDF (optionally downsampled)",
            "requestBody": { "required": true, "content": { "application/json": { "schema": s_ecdf_in } } },
            "responses": { "200": { "description": "OK", "content": { "application/json": { "schema": s_ecdf_out } } } }
          }
        },

        // --- QQ Normal ---
        "/api/v1/stats/qq-normal": {
          "post": {
            "summary": "QQ-plot data against Normal reference (with μ, σ estimates)",
            "requestBody": { "required": true, "content": { "application/json": { "schema": s_qq_in } } },
            "responses": { "200": { "description": "OK", "content": { "application/json": { "schema": s_qq_out } } } }
          }
        },

        // --- Correlation matrix ---
        "/api/v1/stats/corr-matrix": {
          "post": {
            "summary": "Correlation matrix for multiple series",
            "requestBody": { "required": true, "content": { "application/json": { "schema": s_corr_in } } },
            "responses": { "200": { "description": "OK", "content": { "application/json": { "schema": s_corr_out } } } }
          }
        },

        // --- Outliers ---
        "/api/v1/stats/outliers": {
          "post": {
            "summary": "Outlier detection (IQR, z-score, etc.)",
            "requestBody": { "required": true, "content": { "application/json": { "schema": s_outliers_in } } },
            "responses": { "200": { "description": "OK", "content": { "application/json": { "schema": s_outliers_out } } } }
          }
        },

        // --- Normalize ---
        "/api/v1/stats/normalize": {
          "post": {
            "summary": "Normalize vector (z-score or min–max range)",
            "requestBody": { "required": true, "content": { "application/json": { "schema": s_norm_in } } },
            "responses": { "200": { "description": "OK", "content": { "application/json": { "schema": s_norm_out } } } }
          }
        },

        // --- Bin rule ---
        "/api/v1/stats/binrule": {
          "post": {
            "summary": "Pick number of histogram bins via rule",
            "requestBody": { "required": true, "content": { "application/json": { "schema": s_binrule_in } } },
            "responses": { "200": { "description": "OK", "content": { "application/json": { "schema": s_binrule_out } } } }
          }
        }
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

pub async fn stats_summary(Json(inp): Json<SummaryIn>) -> Json<SummaryOut> {
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

    #[inline]
    fn o(x: f64) -> Option<f64> {
        if x.is_nan() { None } else { Some(x) }
    }

    Json(SummaryOut {
        count: n,
        mean: o(m),
        median: o(med),
        std: o(stdv),
        min: o(mn),
        max: o(mx),
        iqr: o(i),
        mad: o(md),
    })
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

    #[inline]
    fn o(x: f64) -> Option<f64> {
        if x.is_nan() { None } else { Some(x) }
    }

    Json(DistOut {
        counts,
        edges,
        quantiles,
        skewness: o(sk),
        excess_kurtosis: o(ek),
        entropy_bits: o(h),
    })
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

    #[inline]
    fn o(x: f64) -> Option<f64> {
        if x.is_nan() { None } else { Some(x) }
    }

    Json(PairOut {
        covariance: o(cov),
        pearson: o(p),
        spearman: o(s),
        kendall: o(k),
    })
}

// ---------------- Optional Prometheus stub ----------------

pub async fn prom_metrics() -> &'static str {
    "# HELP dummy 1\n# TYPE dummy counter\ndummy 1\n"
}

// ========================= ECDF =========================

pub async fn stats_ecdf(Json(inp): Json<EcdfIn>) -> Json<EcdfOut> {
    let mut xs = inp
        .values
        .into_iter()
        .filter(|v| v.is_finite())
        .collect::<Vec<_>>();
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if xs.is_empty() {
        return Json(EcdfOut {
            xs: vec![],
            ps: vec![],
        });
    }

    // compress duplicates into unique x with last index for CDF
    let n = xs.len();
    let mut uniq_x = Vec::with_capacity(n);
    let mut ps = Vec::with_capacity(n);
    let mut i = 0usize;
    while i < n {
        let x = xs[i];
        let mut j = i + 1;
        while j < n && xs[j] == x {
            j += 1;
        }
        // probability at this step = j / n
        uniq_x.push(x);
        ps.push(j as f64 / n as f64);
        i = j;
    }

    // (optional) downsample to ~max_points
    if let Some(max_pts) = inp.max_points.filter(|&m| m > 1 && uniq_x.len() > m) {
        let step = (uniq_x.len() as f64 / max_pts as f64).ceil() as usize;
        let mut dx = Vec::with_capacity(max_pts);
        let mut dp = Vec::with_capacity(max_pts);
        let mut k = 0usize;
        while k < uniq_x.len() {
            dx.push(uniq_x[k]);
            dp.push(ps[k]);
            k = k.saturating_add(step);
        }
        // ensure last point included
        if *dx.last().unwrap() != *uniq_x.last().unwrap() {
            dx.push(*uniq_x.last().unwrap());
            dp.push(*ps.last().unwrap());
        }
        return Json(EcdfOut { xs: dx, ps: dp });
    }

    Json(EcdfOut { xs: uniq_x, ps })
}

// ========================= QQ (Normal) =========================

// Fast probit approximation (Acklam). Max abs error ~ 1e-9 on (0,1).
fn norm_inv(p: f64) -> f64 {
    // guard
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }

    // Coefficients for central region
    const A: [f64; 6] = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_459_239e0,
    ];

    const B: [f64; 5] = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];

    // tails
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838e0,
        -2.549_732_539_343_734e0,
        4.374_664_141_464_968e0,
        2.938_163_982_698_783e0,
    ];

    const D: [f64; 4] = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996e0,
        3.754_408_661_907_416e0,
    ];

    // Break-points
    const P_LOW: f64 = 0.02425;
    const P_HIGH: f64 = 1.0 - P_LOW;

    if p < P_LOW {
        // lower tail
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p <= P_HIGH {
        // central
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        // upper tail
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    }
}

pub async fn stats_qq_normal(Json(inp): Json<QqIn>) -> Json<QqOut> {
    let mut xs = inp
        .values
        .into_iter()
        .filter(|v| v.is_finite())
        .collect::<Vec<_>>();
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = xs.len();
    if n == 0 {
        return Json(QqOut {
            sample_quantiles: vec![],
            theoretical_quantiles: vec![],
            mu_hat: f64::NAN,
            sigma_hat: f64::NAN,
        });
    }

    let robust = inp.robust.unwrap_or(false);
    let (mu, sigma) = if robust {
        let med = median(&xs);
        let madv = mad(&xs); // your robust MAD
        // consistency constant for normal
        (med, 1.4826 * madv.max(1e-12))
    } else {
        let mu = mean(&xs);
        let sd = sample_std_dev(&xs, mu).max(1e-12);
        (mu, sd)
    };

    // Theoretical quantiles for p_i = (i - 0.5)/n
    let mut theor = Vec::with_capacity(n);
    for i in 1..=n {
        let p = (i as f64 - 0.5) / n as f64;
        theor.push(mu + sigma * norm_inv(p));
    }

    Json(QqOut {
        sample_quantiles: xs,
        theoretical_quantiles: theor,
        mu_hat: mu,
        sigma_hat: sigma,
    })
}

// ========================= Correlation Matrix =========================

pub async fn stats_corr_matrix(Json(inp): Json<CorrMatrixIn>) -> Json<CorrMatrixOut> {
    let m = inp.series.len();
    if m == 0 {
        return Json(CorrMatrixOut {
            size: 0,
            names: None,
            matrix: vec![],
        });
    }
    let method = inp.method.unwrap_or(CorrMethod::Pearson);
    let mut mat = vec![0.0f64; m * m];

    for i in 0..m {
        mat[i * m + i] = 1.0;
        for j in (i + 1)..m {
            let v = match method {
                CorrMethod::Pearson => pearson_correlation(&inp.series[i], &inp.series[j]),
                CorrMethod::Spearman => spearman_rho(&inp.series[i], &inp.series[j]),
                CorrMethod::Kendall => kendall_tau_b(&inp.series[i], &inp.series[j]),
            };
            let v = if v.is_nan() { 0.0 } else { v };
            mat[i * m + j] = v;
            mat[j * m + i] = v;
        }
    }

    Json(CorrMatrixOut {
        size: m,
        names: inp.names,
        matrix: mat,
    })
}

// ========================= Outliers =========================

pub async fn stats_outliers(Json(inp): Json<OutliersIn>) -> Json<OutliersOut> {
    let xs = inp
        .values
        .into_iter()
        .filter(|v| v.is_finite())
        .collect::<Vec<_>>();
    if xs.is_empty() {
        return Json(OutliersOut {
            indices: vec![],
            values: vec![],
        });
    }
    let method = inp.method.unwrap_or(OutlierMethod::Iqr);
    let thr = inp.threshold.unwrap_or(3.0); // for zscore default

    let mut idx = Vec::<usize>::new();
    let mut vals = Vec::<f64>::new();

    match method {
        OutlierMethod::Zscore => {
            let mu = mean(&xs);
            let sd = sample_std_dev(&xs, mu).max(1e-12);
            for (i, &x) in xs.iter().enumerate() {
                let z = (x - mu) / sd;
                if z.abs() >= thr {
                    idx.push(i);
                    vals.push(x);
                }
            }
        }
        OutlierMethod::Iqr => {
            let q1 = quantile(&xs, 0.25);
            let q3 = quantile(&xs, 0.75);
            let iqr_v = q3 - q1;
            let lo = q1 - 1.5 * iqr_v;
            let hi = q3 + 1.5 * iqr_v;
            for (i, &x) in xs.iter().enumerate() {
                if x < lo || x > hi {
                    idx.push(i);
                    vals.push(x);
                }
            }
        }
    }

    Json(OutliersOut {
        indices: idx,
        values: vals,
    })
}

// ========================= Normalize =========================

pub async fn stats_normalize(Json(inp): Json<NormalizeIn>) -> Json<NormalizeOut> {
    let xs = inp
        .values
        .into_iter()
        .filter(|v| v.is_finite())
        .collect::<Vec<_>>();
    if xs.is_empty() {
        return Json(NormalizeOut { values: vec![] });
    }
    let method = inp.method.unwrap_or(NormMethod::Zscore);
    let out = match method {
        NormMethod::Zscore => {
            let mu = mean(&xs);
            let sd = sample_std_dev(&xs, mu).max(1e-12);
            xs.iter().map(|&x| (x - mu) / sd).collect::<Vec<_>>()
        }
        NormMethod::Minmax => {
            let a = inp.range.unwrap_or((0.0, 1.0));
            let (lo, hi) = (min(&xs), max(&xs));
            let denom = (hi - lo).max(1e-12);
            xs.iter()
                .map(|&x| a.0 + (x - lo) * (a.1 - a.0) / denom)
                .collect::<Vec<_>>()
        }
    };
    Json(NormalizeOut { values: out })
}

// ========================= Bin Rule =========================

pub async fn stats_binrule(Json(inp): Json<BinRuleIn>) -> Json<BinRuleOut> {
    let xs = inp
        .values
        .into_iter()
        .filter(|v| v.is_finite())
        .collect::<Vec<_>>();
    let n = xs.len();
    if n == 0 {
        return Json(BinRuleOut { bins: 0 });
    }
    let rule = inp
        .rule
        .unwrap_or_else(|| "auto".to_string())
        .to_lowercase();

    // helpers
    let sturges = || (1.0 + (n as f64).log2()).round().max(2.0) as usize;
    let scott = || {
        let mu = mean(&xs);
        let sd = sample_std_dev(&xs, mu).max(1e-12);
        let h = 3.5 * sd / (n as f64).powf(1.0 / 3.0);
        let (lo, hi) = (min(&xs), max(&xs));
        (((hi - lo) / h).ceil() as usize).max(2)
    };
    let fd = || {
        let iqr_v = {
            let q1 = quantile(&xs, 0.25);
            let q3 = quantile(&xs, 0.75);
            q3 - q1
        }
        .max(1e-12);
        let h = 2.0 * iqr_v / (n as f64).powf(1.0 / 3.0);
        let (lo, hi) = (min(&xs), max(&xs));
        (((hi - lo) / h).ceil() as usize).max(2)
    };

    let bins = match rule.as_str() {
        "sturges" => sturges(),
        "scott" => scott(),
        "fd" | "freedmandiaconis" | "freedman_diaconis" => fd(),
        "auto" => {
            // heuristic: max of Sturges and FD, fallback to Scott if degenerate
            let b = sturges().max(fd());
            if b > 0 { b } else { scott() }
        }
        _ => {
            // same heuristic as "auto" (or choose a simpler default if you prefer)
            let b = sturges().max(fd());
            if b > 0 { b } else { scott() }
        }
    };

    Json(BinRuleOut { bins })
}
