//! /stats/distribution

use crate::{
    stats::prelude::*,
    types::{DistIn, DistOut},
};
use axum::Json;

/// Derive histogram, quantiles, and shape statistics (skew, kurtosis, entropy).
///
/// - **Bins**: defaults to 10, min 2
/// - **Quantiles**: defaults to `[0.25, 0.5, 0.75]`
/// - **Edge cases**: when range is degenerate, all mass in first bin
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
    let quantiles = qs.into_iter().map(|p| (p, quantile(&values, p))).collect();

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
