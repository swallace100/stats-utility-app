//! /stats/qq-normal

use crate::{
    stats::prelude::*,
    types::{QqIn, QqOut},
};
use axum::Json;

/// Inverse standard normal CDF (probit) via Acklam's approximation.
///
/// - Max abs error ~ 1e-9 on `(0,1)`
/// - Returns ±∞ for p=0/1 (guarded)
fn norm_inv(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }

    const A: [f64; 6] = [
        -3.969683028665376e1,
        2.209460984245205e2,
        -2.759285104469687e2,
        1.38357751867269e2,
        -3.066479806614716e1,
        2.506628277459239e0,
    ];
    const B: [f64; 5] = [
        -5.447609879822406e1,
        1.615858368580409e2,
        -1.556989798598866e2,
        6.680131188771972e1,
        -1.328068155288572e1,
    ];
    const C: [f64; 6] = [
        -7.784894002430293e-3,
        -3.223964580411365e-1,
        -2.400758277161838e0,
        -2.549732539343734e0,
        4.374664141464968e0,
        2.938163982698783e0,
    ];
    const D: [f64; 4] = [
        7.784695709041462e-3,
        3.224671290700398e-1,
        2.445134137142996e0,
        3.754408661907416e0,
    ];
    const P_LOW: f64 = 0.02425;
    const P_HIGH: f64 = 1.0 - P_LOW;

    if p < P_LOW {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p <= P_HIGH {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    }
}

/// Produce Q–Q plot data against a Normal reference, with μ̂/σ̂ estimates.
///
/// - `robust=true` uses median/MAD (scaled by 1.4826)
/// - `robust=false` (default) uses mean/sample-std
///
/// Returns theoretical quantiles for `p_i=(i-0.5)/n` and the sorted sample.
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
        let madv = mad(&xs);
        (med, 1.4826 * madv.max(1e-12))
    } else {
        let mu = mean(&xs);
        let sd = sample_std_dev(&xs, mu).max(1e-12);
        (mu, sd)
    };

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
