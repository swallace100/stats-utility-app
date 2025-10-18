//! /stats/binrule

use crate::{
    stats::prelude::*,
    types::{BinRuleIn, BinRuleOut},
};
use axum::Json;

/// Choose a histogram bin count using a named rule (`sturges`, `scott`, `fd`, `auto`).
///
/// - `auto` = `max(Sturges, FD)` with Scott fallback on degeneracy
/// - Returns `0` bins for empty input
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

    let sturges = || (1.0 + (n as f64).log2()).round().max(2.0) as usize;
    let scott = || {
        let mu = mean(&xs);
        let sd = sample_std_dev(&xs, mu).max(1e-12);
        let h = 3.5 * sd / (n as f64).powf(1.0 / 3.0);
        let (lo, hi) = (min(&xs), max(&xs));
        (((hi - lo) / h).ceil() as usize).max(2)
    };
    let fd = || {
        let q1 = quantile(&xs, 0.25);
        let q3 = quantile(&xs, 0.75);
        let iqr_v = (q3 - q1).max(1e-12);
        let h = 2.0 * iqr_v / (n as f64).powf(1.0 / 3.0);
        let (lo, hi) = (min(&xs), max(&xs));
        (((hi - lo) / h).ceil() as usize).max(2)
    };

    let bins = match rule.as_str() {
        "sturges" => sturges(),
        "scott" => scott(),
        "fd" | "freedmandiaconis" | "freedman_diaconis" => fd(),
        "auto" => {
            let b = sturges().max(fd());
            if b > 0 { b } else { scott() }
        }
        _ => {
            let b = sturges().max(fd());
            if b > 0 { b } else { scott() }
        }
    };

    Json(BinRuleOut { bins })
}
