//! /stats/ecdf

use crate::types::{EcdfIn, EcdfOut};
use axum::Json;

/// Compute empirical CDF (ECDF), with optional downsampling for large outputs.
///
/// - Input NaN/Inf are filtered out.
/// - Output `(xs, ps)` are unique sorted values and their cumulative probabilities.
/// - If `max_points` is set, the output is downsampled uniformly (end point preserved).
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
        uniq_x.push(x);
        ps.push(j as f64 / n as f64);
        i = j;
    }

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
        if *dx.last().unwrap() != *uniq_x.last().unwrap() {
            dx.push(*uniq_x.last().unwrap());
            dp.push(*ps.last().unwrap());
        }
        return Json(EcdfOut { xs: dx, ps: dp });
    }

    Json(EcdfOut { xs: uniq_x, ps })
}
