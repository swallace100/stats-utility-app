//! /stats/normalize

use crate::{
    stats::prelude::*,
    types::{NormMethod, NormalizeIn, NormalizeOut},
};
use axum::Json;

/// Normalize a numeric vector using Z-score or min–max scaling.
///
/// - Defaults to `Zscore`
/// - Min–max range defaults to `(0.0, 1.0)`
/// - Non-finite inputs are filtered out before normalization
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
