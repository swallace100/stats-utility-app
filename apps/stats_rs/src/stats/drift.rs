use crate::stats::prelude::*;

/// Population Stability Index (PSI) comparing actual vs. expected distributions
/// by binning using expected quantiles. Larger PSI → bigger drift.
/// Common rule of thumb: <0.1 small; 0.1–0.25 moderate; >0.25 large.
pub fn psi_quantile_bins(expected: &[f64], actual: &[f64], bins: usize) -> f64 {
    assert!(bins >= 2);
    if expected.is_empty() || actual.is_empty() {
        return f64::NAN;
    }

    // Build bin edges from expected quantiles
    let mut edges = Vec::with_capacity(bins + 1);
    for i in 0..=bins {
        let p = i as f64 / bins as f64;
        edges.push(quantile(expected, p));
    }

    // Count into bins
    let mut ce = vec![0usize; bins];
    let mut ca = vec![0usize; bins];

    let bin_of = |x: f64, edges: &[f64]| -> usize {
        // rightmost inclusive
        let mut lo = 0usize;
        let mut hi = edges.len() - 1;
        if x <= edges[0] {
            return 0;
        }
        if x >= edges[hi] {
            return hi - 1;
        }
        while lo + 1 < hi {
            let mid = (lo + hi) / 2;
            if x <= edges[mid] {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        lo
    };

    for &x in expected {
        ce[bin_of(x, &edges)] += 1;
    }
    for &x in actual {
        ca[bin_of(x, &edges)] += 1;
    }

    let ne = expected.len() as f64;
    let na = actual.len() as f64;
    let eps = 1e-12;

    let mut psi = 0.0;
    for i in 0..bins {
        let pe = (ce[i] as f64 / ne).max(eps);
        let pa = (ca[i] as f64 / na).max(eps);
        psi += (pa - pe) * (pa / pe).ln();
    }
    psi
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::utils::EPS_TIGHT;

    #[test]
    fn psi_drift() {
        // Identical distributions → PSI ≈ 0
        let expected = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let actual_same = expected.clone();
        let psi0 = psi_quantile_bins(&expected, &actual_same, 5);
        assert!(psi0.abs() < EPS_TIGHT);

        // Shifted distribution → PSI > 0
        let actual_shift = vec![2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0];
        let psi = psi_quantile_bins(&expected, &actual_shift, 5);
        assert!(psi > 0.0);
    }
}

#[cfg(test)]
mod more_tests {
    use super::*;

    #[test]
    #[should_panic]
    fn psi_bins_must_be_at_least_two() {
        let _ = psi_quantile_bins(&[1.0, 2.0], &[1.0, 2.0], 1);
    }

    #[test]
    fn psi_empty_inputs_nan() {
        assert!(psi_quantile_bins(&[], &[1.0], 5).is_nan());
        assert!(psi_quantile_bins(&[1.0], &[], 5).is_nan());
    }

    #[test]
    fn psi_counts_sum_to_lengths() {
        // Mirror the inner counting logic to ensure bins cover domain properly.
        let expected = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let actual = vec![0.0, 0.0, 5.0, 6.0, 7.0, 8.0];
        let bins = 4;

        // Rebuild edges exactly like psi_quantile_bins does
        let mut edges = Vec::with_capacity(bins + 1);
        for i in 0..=bins {
            edges.push(quantile(&expected, i as f64 / bins as f64));
        }

        let mut ce = vec![0usize; bins];
        let mut ca = vec![0usize; bins];

        let bin_of = |x: f64, edges: &[f64]| -> usize {
            let mut lo = 0usize;
            let mut hi = edges.len() - 1;
            if x <= edges[0] {
                return 0;
            }
            if x >= edges[hi] {
                return hi - 1;
            }
            while lo + 1 < hi {
                let mid = (lo + hi) / 2;
                if x <= edges[mid] {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            lo
        };

        for &x in &expected {
            ce[bin_of(x, &edges)] += 1;
        }
        for &x in &actual {
            ca[bin_of(x, &edges)] += 1;
        }

        assert_eq!(ce.iter().sum::<usize>(), expected.len());
        assert_eq!(ca.iter().sum::<usize>(), actual.len());

        // And the PSI computed is finite
        let psi = psi_quantile_bins(&expected, &actual, bins);
        assert!(psi.is_finite());
        assert!(psi > 0.0); // clearly shifted right
    }

    #[test]
    fn psi_all_mass_to_upper_tail_is_positive() {
        // Expected roughly uniform in [0,1]; actual entirely at 1.0
        let expected = vec![0.0, 0.25, 0.5, 0.75, 1.0, 0.1, 0.6, 0.8, 0.2, 0.4];
        let actual = vec![1.0; 10];
        let psi = psi_quantile_bins(&expected, &actual, 5);
        assert!(psi.is_finite());
        assert!(psi > 0.0);
    }

    #[test]
    fn psi_degenerate_expected_edges_are_handled() {
        // All expected values are identical → all quantile edges equal.
        // Implementation should still produce a finite PSI thanks to epsilon clamps.
        let expected = vec![5.0; 20];
        let actual = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let psi = psi_quantile_bins(&expected, &actual, 5);
        assert!(psi.is_finite());
        assert!(psi >= 0.0);
    }

    #[test]
    fn psi_identical_distributions_is_near_zero_small_eps() {
        // Another sanity identical-dists case with different bins
        let xs: Vec<f64> = (1..=50).map(|i| i as f64).collect();
        let psi = psi_quantile_bins(&xs, &xs, 10);
        assert!(psi.abs() < 1e-9);
    }
}
