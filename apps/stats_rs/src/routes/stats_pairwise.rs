//! /stats/pairwise

use crate::{
    stats::prelude::*,
    types::{PairIn, PairOut},
};
use axum::Json;

/// Compute covariance and correlations (Pearson, Spearman, Kendall) for two vectors.
///
/// Returns `None` metrics if lengths mismatch or vectors are empty.
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
