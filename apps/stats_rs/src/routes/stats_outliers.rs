//! /stats/outliers

use crate::{
    stats::prelude::*,
    types::{OutlierMethod, OutliersIn, OutliersOut},
};
use axum::Json;

/// Detect outliers via Z-score or IQR rules.
///
/// - `method` defaults to IQR
/// - `threshold` (Z-score) defaults to `3.0`
/// - Non-finite inputs are filtered out
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
    let thr = inp.threshold.unwrap_or(3.0);

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
