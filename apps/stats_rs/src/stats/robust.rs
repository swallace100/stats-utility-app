use crate::stats::prelude::*;

pub fn mad(xs: &[f64]) -> f64 {
    // Median Absolute Deviation (about the median)
    if xs.is_empty() {
        return f64::NAN;
    }
    let med = super::median(xs);
    let devs: Vec<f64> = xs.iter().map(|&x| (x - med).abs()).collect();
    super::median(&devs)
}

/// Robust z-score using MAD (≈ 1.4826 * MAD to estimate sigma)
pub fn robust_zscores_mad(xs: &[f64]) -> Vec<f64> {
    if xs.is_empty() {
        return vec![];
    }
    let med = median(xs);
    let devs: Vec<f64> = xs.iter().map(|&x| (x - med).abs()).collect();
    let mad = median(&devs);
    let scale = 1.4826_f64 * mad;
    xs.iter()
        .map(|&x| if scale == 0.0 { 0.0 } else { (x - med) / scale })
        .collect()
}

/// Trimmed mean keeping central proportion keep in (0,1]. E.g., keep=0.9 trims 5% each tail.
pub fn trimmed_mean(xs: &[f64], keep: f64) -> f64 {
    assert!((0.0..=1.0).contains(&keep));
    if xs.is_empty() {
        return f64::NAN;
    }
    if keep == 1.0 {
        return mean(xs);
    }
    if keep == 0.0 {
        return median(xs);
    }
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    let keep_n = (keep * n as f64).round().clamp(1.0, n as f64) as usize;
    let drop = (n - keep_n) / 2;
    mean(&v[drop..drop + keep_n])
}

/// Winsorized mean: cap extremes to given quantiles (e.g., q=0.05).
pub fn winsorized_mean(xs: &[f64], q: f64) -> f64 {
    assert!((0.0..=0.5).contains(&q));
    if xs.is_empty() {
        return f64::NAN;
    }
    let lo = quantile(xs, q);
    let hi = quantile(xs, 1.0 - q);
    let w: Vec<f64> = xs.iter().map(|&x| x.clamp(lo, hi)).collect();
    mean(&w)
}

/// Geometric mean; returns NaN if any value <= 0.
pub fn geometric_mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return f64::NAN;
    }
    if xs.iter().any(|&x| x <= 0.0) {
        return f64::NAN;
    }
    let sum_logs: f64 = xs.iter().map(|&x| x.ln()).sum();
    (sum_logs / xs.len() as f64).exp()
}

/// Harmonic mean; returns NaN if any value <= 0.
pub fn harmonic_mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return f64::NAN;
    }
    if xs.iter().any(|&x| x <= 0.0) {
        return f64::NAN;
    }
    let denom: f64 = xs.iter().map(|&x| 1.0 / x).sum();
    xs.len() as f64 / denom
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approx;
    use crate::stats::utils::EPS_TIGHT;

    #[test]
    fn robust_shape_and_scaling() {
        // Excess kurtosis on a near-uniform spread should be < 0 (platykurtic)
        let xs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ek = excess_kurtosis(&xs);
        assert!(ek.is_finite() && ek < 0.0);

        // Trimmed mean (keep 60% ⇒ keep_n=3 → [2,3,4])
        let out = vec![1.0, 100.0, 2.0, 3.0, 4.0];
        let tm = trimmed_mean(&out, 0.6);
        approx!(tm, 3.0, 1e-9);

        // Winsorized mean with q=0.2 caps to ~6.8 on sorted [1,2,3,4,100]
        let wm = winsorized_mean(&out, 0.2);
        approx!(wm, 6.8, 1e-6);

        // Robust z via MAD on symmetric set
        let xs2 = vec![1.0, 2.0, 3.0, 4.0];
        let rz = robust_zscores_mad(&xs2);
        assert_eq!(rz.len(), 4);
        assert!((rz[0] + 1.0118).abs() < 5e-3);
        assert!((rz[3] - 1.0118).abs() < 5e-3);
    }

    #[test]
    fn alt_means_and_corr_smoke() {
        // geometric & harmonic means
        let pos = vec![1.0, 2.0, 4.0, 8.0];
        approx!(geometric_mean(&pos), 2.8284271247461903, EPS_TIGHT);
        approx!(harmonic_mean(&pos), 2.1333333333333333, EPS_TIGHT);

        // correlation & skewness smoke on a simple linear relation
        let xs = vec![1.0, 2.0, 3.0, 4.0];
        let ys = vec![2.0, 4.0, 6.0, 8.0];
        approx!(covariance(&xs, &ys), 3.3333333333333335, EPS_TIGHT);
        approx!(pearson_correlation(&xs, &ys), 1.0, EPS_TIGHT);
        assert!(skewness(&xs).abs() < EPS_TIGHT);
    }
}

#[cfg(test)]
mod edge_tests {
    use super::*;
    use crate::approx;
    use crate::stats::utils::EPS_TIGHT;

    #[test]
    fn mad_edges() {
        assert!(mad(&[]).is_nan());
        approx!(mad(&[5.0, 5.0, 5.0]), 0.0, EPS_TIGHT); // all equal → MAD=0
        // asymmetric but centered
        approx!(mad(&[1.0, 2.0, 100.0]), 1.0, 1e-12); // median=2 → |devs|=1,0,98 → median=1
    }

    #[test]
    fn robust_zscores_mad_edges_and_constants() {
        let empty: Vec<f64> = vec![];
        let z_empty = robust_zscores_mad(&empty);
        assert!(z_empty.is_empty());

        let consts = vec![3.0, 3.0, 3.0, 3.0];
        let zc = robust_zscores_mad(&consts);
        assert_eq!(zc.len(), consts.len());
        assert!(zc.iter().all(|&v| v.abs() <= EPS_TIGHT)); // scale==0 → zeros
    }

    #[test]
    fn trimmed_mean_boundaries_and_empty() {
        let xs = vec![1.0, 2.0, 3.0, 4.0];

        // keep=1.0 → ordinary mean
        approx!(trimmed_mean(&xs, 1.0), mean(&xs), EPS_TIGHT);

        // keep=0.0 → keep one central element (rounded+clamped) → median
        approx!(trimmed_mean(&xs, 0.0), median(&xs), EPS_TIGHT);

        // odd/even check: centered block for keep around 0.5
        approx!(trimmed_mean(&xs, 0.5), 2.5, EPS_TIGHT); // keep_n=2 → [2,3]

        assert!(trimmed_mean(&[], 0.5).is_nan());
    }

    #[test]
    fn winsorized_mean_boundaries_and_empty() {
        let xs = vec![1.0, 2.0, 3.0, 4.0, 100.0];

        // q=0.0 → no capping → mean
        approx!(winsorized_mean(&xs, 0.0), mean(&xs), EPS_TIGHT);

        // q=0.5 → both quantiles at median → everything clamped to median → mean==median
        let med = median(&xs);
        approx!(winsorized_mean(&xs, 0.5), med, EPS_TIGHT);

        // monotonic: winsorized mean should lie within [min,max] and between mean & median for this skewed set
        let wm = winsorized_mean(&xs, 0.2);
        assert!(wm >= min(&xs) && wm <= max(&xs));
        // sanity: for this right-skew, winsorizing should pull mean toward median
        assert!(wm < mean(&xs) && wm > med);

        assert!(winsorized_mean(&[], 0.2).is_nan());
    }

    #[test]
    fn geometric_and_harmonic_mean_edges() {
        // empty → NaN
        assert!(geometric_mean(&[]).is_nan());
        assert!(harmonic_mean(&[]).is_nan());

        // non-positive → NaN by design
        assert!(geometric_mean(&[1.0, 0.0, 2.0]).is_nan());
        assert!(geometric_mean(&[1.0, -2.0]).is_nan());
        assert!(harmonic_mean(&[1.0, 0.0, 2.0]).is_nan());
        assert!(harmonic_mean(&[1.0, -2.0]).is_nan());

        // well-defined positive inputs
        let pos = vec![1.0, 2.0, 4.0, 8.0];
        approx!(geometric_mean(&pos), 2.8284271247461903, EPS_TIGHT);
        approx!(harmonic_mean(&pos), 2.1333333333333333, EPS_TIGHT);
    }
}
