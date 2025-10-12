use crate::stats::prelude::*;

/// Sample covariance (denominator n-1). xs, ys must have same length >= 2.
pub fn covariance(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len();
    assert_eq!(n, ys.len(), "xs and ys must have same length");
    if n < 2 {
        return f64::NAN;
    }
    let mx = super::mean(xs);
    let my = super::mean(ys);
    let mut s = 0.0;
    for i in 0..n {
        s += (xs[i] - mx) * (ys[i] - my);
    }
    s / (n as f64 - 1.0)
}

/// Pearson correlation coefficient r (sample version).
pub fn pearson_correlation(xs: &[f64], ys: &[f64]) -> f64 {
    let cov = covariance(xs, ys);
    let sx = super::sample_std_dev(xs, super::mean(xs));
    let sy = super::sample_std_dev(ys, super::mean(ys));
    cov / (sx * sy)
}

/// Spearman's rho (Pearson correlation of average ranks).
pub fn spearman_rho(xs: &[f64], ys: &[f64]) -> f64 {
    assert_eq!(xs.len(), ys.len());
    let rx = average_ranks(xs);
    let ry = average_ranks(ys);
    pearson_correlation(&rx, &ry)
}

/// Kendall's tau-b (tie-aware). Returns NaN if len < 2.
pub fn kendall_tau_b(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len();
    assert_eq!(n, ys.len());
    if n < 2 {
        return f64::NAN;
    }

    // Rank with average ties
    let rx = average_ranks(xs);
    let ry = average_ranks(ys);

    // Count concordant/discordant; O(n^2) but fine for evals.
    let mut c = 0_i64;
    let mut d = 0_i64;
    let mut tx = 0_i64; // ties in x only
    let mut ty = 0_i64; // ties in y only

    for i in 0..n {
        for j in (i + 1)..n {
            let dx = rx[i].partial_cmp(&rx[j]).unwrap();
            let dy = ry[i].partial_cmp(&ry[j]).unwrap();
            match (dx, dy) {
                (std::cmp::Ordering::Less, std::cmp::Ordering::Less)
                | (std::cmp::Ordering::Greater, std::cmp::Ordering::Greater) => c += 1,
                (std::cmp::Ordering::Less, std::cmp::Ordering::Greater)
                | (std::cmp::Ordering::Greater, std::cmp::Ordering::Less) => d += 1,
                (std::cmp::Ordering::Equal, std::cmp::Ordering::Equal) => { /* tied pair in both → ignored */
                }
                (std::cmp::Ordering::Equal, _) => tx += 1,
                (_, std::cmp::Ordering::Equal) => ty += 1,
            }
        }
    }

    let num = (c - d) as f64;
    let den = (((c + d + tx) as f64) * ((c + d + ty) as f64)).sqrt();
    if den == 0.0 { f64::NAN } else { num / den }
}

/// Sample skewness (Fisher–Pearson adjusted).
pub fn skewness(xs: &[f64]) -> f64 {
    let n = xs.len();
    if n < 3 {
        return f64::NAN;
    }
    let m = super::mean(xs);
    let s = super::sample_std_dev(xs, m);
    if s == 0.0 {
        return 0.0;
    }
    let m3: f64 = xs.iter().map(|&x| ((x - m) / s).powi(3)).sum::<f64>();
    (n as f64) * m3 / ((n as f64 - 1.0) * (n as f64 - 2.0))
}

/// Excess kurtosis (Fisher, 0 for normal). Uses sample correction.
pub fn excess_kurtosis(xs: &[f64]) -> f64 {
    let n = xs.len();
    if n < 4 {
        return f64::NAN;
    }
    let m = mean(xs);
    let s = sample_std_dev(xs, m);
    if s == 0.0 {
        return f64::NAN;
    }
    let m4: f64 = xs.iter().map(|&x| ((x - m) / s).powi(4)).sum::<f64>() / n as f64;
    // unbiased-ish estimator (Fisher) correction
    let num = (n as f64) * (n as f64 + 1.0) * (m4 - 3.0) + 6.0;
    let den = (n as f64 - 1.0) * (n as f64 - 2.0) * (n as f64 - 3.0);
    num / den
}

/// Average ranks (handles ties). Returns ranks aligned with xs.
pub fn average_ranks(xs: &[f64]) -> Vec<f64> {
    let n = xs.len();
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&i, &j| xs[i].partial_cmp(&xs[j]).unwrap());
    let mut ranks = vec![0.0; n];
    let mut i = 0;
    while i < n {
        let mut j = i + 1;
        while j < n && xs[idx[i]] == xs[idx[j]] {
            j += 1;
        }
        let avg = (i + 1 + j) as f64 / 2.0;
        for k in i..j {
            ranks[idx[k]] = avg;
        }
        i = j;
    }
    ranks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approx; // macro from utils.rs via #[macro_export]
    use crate::stats::utils::{EPS, EPS_TIGHT};

    #[test]
    fn ranks_and_correlations() {
        // average ranks with ties
        let a = vec![10.0, 10.0, 30.0];
        let r = average_ranks(&a);
        approx!(r[0], 1.5, EPS);
        approx!(r[1], 1.5, EPS);
        approx!(r[2], 3.0, EPS);

        // perfect and inverse orders
        let x = vec![1.0, 2.0, 3.0, 4.0];
        let y = vec![10.0, 20.0, 30.0, 40.0];
        let y_inv = vec![40.0, 30.0, 20.0, 10.0];

        approx!(spearman_rho(&x, &y), 1.0, EPS);
        approx!(spearman_rho(&x, &y_inv), -1.0, EPS);

        approx!(kendall_tau_b(&x, &y), 1.0, EPS);
        approx!(kendall_tau_b(&x, &y_inv), -1.0, EPS);
    }

    #[test]
    fn corr_and_skewness_smoke() {
        let xs = vec![1.0, 2.0, 3.0, 4.0];
        let ys = vec![2.0, 4.0, 6.0, 8.0];

        // sample covariance for perfect linear relation (n-1 denom) = 10/3
        approx!(covariance(&xs, &ys), 3.3333333333333335, EPS_TIGHT);
        approx!(pearson_correlation(&xs, &ys), 1.0, EPS_TIGHT);

        // symmetric data → skewness ≈ 0
        assert!(skewness(&xs).abs() < EPS_TIGHT);
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;
    use crate::approx;
    use crate::stats::utils::EPS_TIGHT;

    // --- length mismatch panics ---
    #[test]
    #[should_panic]
    fn covariance_len_mismatch_panics() {
        let _ = covariance(&[1.0, 2.0], &[1.0]);
    }

    #[test]
    #[should_panic]
    fn pearson_len_mismatch_panics() {
        let _ = pearson_correlation(&[1.0], &[1.0, 2.0]);
    }

    #[test]
    #[should_panic]
    fn spearman_len_mismatch_panics() {
        let _ = spearman_rho(&[1.0], &[1.0, 2.0]);
    }

    #[test]
    #[should_panic]
    fn kendall_len_mismatch_panics() {
        let _ = kendall_tau_b(&[1.0], &[1.0, 2.0]);
    }

    // --- small n / constants ---
    #[test]
    fn covariance_and_kendall_small_n_nan() {
        assert!(covariance(&[1.0], &[2.0]).is_nan());
        assert!(kendall_tau_b(&[1.0], &[2.0]).is_nan());
    }

    #[test]
    fn pearson_and_spearman_constant_vectors_nan() {
        // std == 0 → pearson NaN
        let xs = vec![3.0, 3.0, 3.0];
        let ys = vec![1.0, 2.0, 3.0];
        assert!(pearson_correlation(&xs, &ys).is_nan());

        // Spearman ranks are all equal on xs → std=0 → NaN
        let xs2 = vec![7.0, 7.0, 7.0];
        let ys2 = vec![10.0, 20.0, 30.0];
        assert!(spearman_rho(&xs2, &ys2).is_nan());
    }

    #[test]
    fn skewness_small_n_and_constant_behavior() {
        // n < 3 → NaN
        assert!(skewness(&[1.0, 2.0]).is_nan());
        // std == 0 → defined as 0.0 in this impl
        approx!(skewness(&[5.0, 5.0, 5.0]), 0.0, EPS_TIGHT);
    }

    // --- negative correlation sanity ---
    #[test]
    fn pearson_negative_one_on_inverse_linear() {
        let x = vec![1.0, 2.0, 3.0, 4.0];
        let y_inv = vec![40.0, 30.0, 20.0, 10.0];
        approx!(pearson_correlation(&x, &y_inv), -1.0, EPS_TIGHT);
    }

    // --- average_ranks alignment & tie blocks ---
    #[test]
    fn average_ranks_alignment_and_multitied_block() {
        // Values: [3,3,3,1] → ranks: [3,3,3,1] aligned to original indices
        let xs = vec![3.0, 3.0, 3.0, 1.0];
        let r = average_ranks(&xs);
        // The three 3's occupy positions 2..=4 when sorted (1-based),
        // so average rank = (2 + 4) / 2 = 3.0; the '1' gets rank 1.0.
        approx!(r[0], 3.0, EPS_TIGHT);
        approx!(r[1], 3.0, EPS_TIGHT);
        approx!(r[2], 3.0, EPS_TIGHT);
        approx!(r[3], 1.0, EPS_TIGHT);

        // Another tie pattern: [1,2,2,2,5] → ranks [1, 3, 3, 3, 5]
        let ys = vec![1.0, 2.0, 2.0, 2.0, 5.0];
        let ry = average_ranks(&ys);
        approx!(ry[0], 1.0, EPS_TIGHT);
        approx!(ry[1], 3.0, EPS_TIGHT);
        approx!(ry[2], 3.0, EPS_TIGHT);
        approx!(ry[3], 3.0, EPS_TIGHT);
        approx!(ry[4], 5.0, EPS_TIGHT);
    }
}
