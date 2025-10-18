//! /stats/summary

use crate::{
    stats::prelude::*,
    types::{SummaryIn, SummaryOut},
};
use axum::Json;

/// Compute core univariate summary statistics.
///
/// Returns `None` for undefined metrics (e.g., std with `n < 2`).
///
/// - **Request**: [`SummaryIn`]
/// - **Response**: [`SummaryOut`]
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
