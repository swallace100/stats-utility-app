//! /stats/corr-matrix

use crate::{
    stats::prelude::*,
    types::{CorrMatrixIn, CorrMatrixOut, CorrMethod},
};
use axum::Json;

/// Compute an `m×m` correlation matrix across multiple series.
///
/// - `method` defaults to Pearson
/// - Returns a flattened row-major matrix in [`CorrMatrixOut::matrix`]
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
