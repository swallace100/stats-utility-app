pub fn sum(xs: &[f64]) -> f64 {
    xs.iter().copied().sum()
}

pub fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        f64::NAN
    } else {
        sum(xs) / xs.len() as f64
    }
}

pub fn median(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return f64::NAN;
    }
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

/// Returns all modes (handles multimodal data).
pub fn mode(xs: &[f64]) -> Vec<f64> {
    use std::collections::HashMap;
    let mut map: HashMap<i64, (usize, f64)> = HashMap::new();
    // Bucket by rounding to 1e-12 bins to avoid tiny float noise; adjust if needed.
    const SCALE: f64 = 1e12;
    for &x in xs {
        let k = (x * SCALE).round() as i64;
        let e = map.entry(k).or_insert((0, x));
        e.0 += 1;
    }
    let max_f = map.values().map(|(c, _)| *c).max().unwrap_or(0);
    if max_f == 0 {
        return vec![];
    }
    let mut modes: Vec<f64> = map
        .into_iter()
        .filter_map(|(_, (c, val))| if c == max_f { Some(val) } else { None })
        .collect();
    modes.sort_by(|a, b| a.partial_cmp(b).unwrap());
    modes
}

pub fn min(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return f64::NAN;
    }
    xs.iter().copied().fold(f64::INFINITY, f64::min)
}
pub fn max(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return f64::NAN;
    }
    xs.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}
pub fn range(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        f64::NAN
    } else {
        max(xs) - min(xs)
    }
}

pub fn sample_variance(xs: &[f64], mean: f64) -> f64 {
    let n = xs.len();
    if n < 2 {
        return f64::NAN;
    }
    let s: f64 = xs
        .iter()
        .map(|&x| {
            let d = x - mean;
            d * d
        })
        .sum();
    s / (n as f64 - 1.0)
}
pub fn population_variance(xs: &[f64], mean: f64) -> f64 {
    let n = xs.len();
    if n == 0 {
        return f64::NAN;
    }
    let s: f64 = xs
        .iter()
        .map(|&x| {
            let d = x - mean;
            d * d
        })
        .sum();
    s / n as f64
}
pub fn sample_std_dev(xs: &[f64], mean: f64) -> f64 {
    sample_variance(xs, mean).sqrt()
}
pub fn population_std_dev(xs: &[f64], mean: f64) -> f64 {
    population_variance(xs, mean).sqrt()
}

// R-7 quantile
pub fn quantile(xs: &[f64], p: f64) -> f64 {
    assert!((0.0..=1.0).contains(&p), "p must be in [0,1]");
    let n = xs.len();
    if n == 0 {
        return f64::NAN;
    }
    if n == 1 {
        return xs[0];
    }
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let h = (n - 1) as f64 * p;
    let i = h.floor() as usize;
    let j = h.ceil() as usize;
    if i == j {
        v[i]
    } else {
        v[i] + (h - i as f64) * (v[j] - v[i])
    }
}
pub fn quartiles(xs: &[f64]) -> (f64, f64, f64) {
    (quantile(xs, 0.25), quantile(xs, 0.5), quantile(xs, 0.75))
}
pub fn iqr(xs: &[f64]) -> f64 {
    let (q1, _, q3) = quartiles(xs);
    q3 - q1
}

#[cfg(test)]
mod tests {
    use super::*; // items from this module
    use crate::approx;
    use crate::stats::prelude::*; // cross-module stats (covariance, skewness, etc.)
    use crate::stats::utils::EPS_TIGHT; // approx! macro (from utils.rs via #[macro_export])

    #[test]
    fn basics_summary_quantiles() {
        let xs = vec![1.0, 2.0, 3.0, 4.0];

        // basic summaries
        approx!(sum(&xs), 10.0, EPS_TIGHT);
        approx!(min(&xs), 1.0, EPS_TIGHT);
        approx!(max(&xs), 4.0, EPS_TIGHT);
        approx!(range(&xs), 3.0, EPS_TIGHT);

        // variance/std
        let m = mean(&xs);
        let pv = population_variance(&xs, m);
        let sv = sample_variance(&xs, m);
        approx!(pv, 1.25, EPS_TIGHT);
        approx!(sv, 1.6666666666666667, EPS_TIGHT);
        approx!(population_std_dev(&xs, m), pv.sqrt(), EPS_TIGHT);

        // quantiles/IQR/MAD/mode
        approx!(quantile(&xs, 0.25), 1.75, EPS_TIGHT);
        let (q1, q2, q3) = quartiles(&xs);
        approx!(q1, 1.75, EPS_TIGHT);
        approx!(q2, 2.5, EPS_TIGHT);
        approx!(q3, 3.25, EPS_TIGHT);
        approx!(iqr(&xs), 1.5, EPS_TIGHT);

        approx!(mad(&xs), 1.0, EPS_TIGHT);
        assert_eq!(mode(&xs), vec![1.0, 2.0, 3.0, 4.0]);
    }
}

#[cfg(test)]
mod more_tests {
    use super::*;
    use crate::approx;
    use crate::stats::utils::EPS_TIGHT;

    #[test]
    fn empty_and_singleton() {
        let empty: Vec<f64> = vec![];
        assert!(mean(&empty).is_nan());
        assert!(median(&empty).is_nan());
        assert!(min(&empty).is_nan());
        assert!(max(&empty).is_nan());
        assert!(range(&empty).is_nan());
        assert!(sample_variance(&empty, f64::NAN).is_nan());
        assert!(population_variance(&empty, f64::NAN).is_nan());
        assert!(sample_std_dev(&empty, f64::NAN).is_nan());
        assert!(population_std_dev(&empty, f64::NAN).is_nan());
        assert!(quantile(&empty, 0.5).is_nan());
        assert_eq!(mode(&empty), Vec::<f64>::new());

        let one = vec![42.0];
        approx!(mean(&one), 42.0, EPS_TIGHT);
        approx!(median(&one), 42.0, EPS_TIGHT);
        approx!(min(&one), 42.0, EPS_TIGHT);
        approx!(max(&one), 42.0, EPS_TIGHT);
        approx!(range(&one), 0.0, EPS_TIGHT);
        // sample variance undefined for n<2
        assert!(sample_variance(&one, 42.0).is_nan());
        assert!(sample_std_dev(&one, 42.0).is_nan());
        // population variance/std are 0 for n=1
        approx!(population_variance(&one, 42.0), 0.0, EPS_TIGHT);
        approx!(population_std_dev(&one, 42.0), 0.0, EPS_TIGHT);
        approx!(quantile(&one, 0.0), 42.0, EPS_TIGHT);
        approx!(quantile(&one, 1.0), 42.0, EPS_TIGHT);
    }

    #[test]
    fn median_even_and_odd_unsorted() {
        let odd = vec![3.0, 1.0, 2.0];
        approx!(median(&odd), 2.0, EPS_TIGHT);

        let even = vec![4.0, 1.0, 3.0, 2.0];
        approx!(median(&even), 2.5, EPS_TIGHT);
    }

    #[test]
    fn mode_multimodal_and_stability() {
        // two modes → return both, sorted ascending
        let xs = vec![1.0, 2.0, 2.0, 3.0, 3.0];
        let m = mode(&xs);
        assert_eq!(m, vec![2.0, 3.0]);

        // all singletons → all values (sorted)
        let ys = vec![10.0, -1.0, 5.0];
        let m2 = mode(&ys);
        assert_eq!(m2, vec![-1.0, 5.0, 10.0]);

        // tiny float noise buckets into same 1e-12 bin
        let zs = vec![1.0, 1.0 + 1e-13, 2.0];
        let m3 = mode(&zs);
        assert_eq!(m3, vec![1.0]); // 1.* occurs twice after binning
    }

    #[test]
    fn min_max_range_negatives_and_constants() {
        let xs = vec![-5.0, -1.0, 0.0, 2.0];
        let same = vec![3.0, 3.0, 3.0];
        approx!(min(&xs), -5.0, EPS_TIGHT);
        approx!(max(&xs), 2.0, EPS_TIGHT);
        approx!(range(&xs), 7.0, EPS_TIGHT);

        approx!(min(&same), 3.0, EPS_TIGHT);
        approx!(max(&same), 3.0, EPS_TIGHT);
        approx!(range(&same), 0.0, EPS_TIGHT);
    }

    #[test]
    fn variance_std_edge_cases() {
        // sample variance/std undefined for n<2
        let one = vec![7.0];
        assert!(sample_variance(&one, 7.0).is_nan());
        assert!(sample_std_dev(&one, 7.0).is_nan());

        // small dataset sanity
        let xs = vec![1.0, 2.0, 3.0, 4.0];
        let m = mean(&xs);
        approx!(sample_variance(&xs, m), 1.6666666666666667, EPS_TIGHT);
        approx!(population_variance(&xs, m), 1.25, EPS_TIGHT);
        approx!(sample_std_dev(&xs, m), 1.2909944487358056, EPS_TIGHT);
        approx!(population_std_dev(&xs, m), 1.118033988749895, EPS_TIGHT);
    }

    #[test]
    fn quantile_edges_and_interpolation() {
        let xs = vec![1.0, 2.0, 3.0, 4.0]; // sorted by impl
        // edges
        approx!(quantile(&xs, 0.0), 1.0, EPS_TIGHT);
        approx!(quantile(&xs, 1.0), 4.0, EPS_TIGHT);
        // interior interpolation (R-7)
        approx!(quantile(&xs, 0.25), 1.75, EPS_TIGHT);
        approx!(quantile(&xs, 0.50), 2.5, EPS_TIGHT);
        approx!(quantile(&xs, 0.75), 3.25, EPS_TIGHT);
    }

    #[test]
    #[should_panic(expected = "p must be in [0,1]")]
    fn quantile_p_below_zero_panics() {
        let xs = vec![1.0, 2.0, 3.0];
        let _ = quantile(&xs, -0.01);
    }

    #[test]
    #[should_panic(expected = "p must be in [0,1]")]
    fn quantile_p_above_one_panics() {
        let xs = vec![1.0, 2.0, 3.0];
        let _ = quantile(&xs, 1.01);
    }
}
