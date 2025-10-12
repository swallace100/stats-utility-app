use crate::stats::prelude::*;

/// Standard z-scores using sample std. Returns empty vec for empty input.
pub fn zscores(xs: &[f64]) -> Vec<f64> {
    if xs.is_empty() {
        return vec![];
    }
    let m = mean(xs);
    let s = sample_std_dev(xs, m);
    xs.iter()
        .map(|&x| if s == 0.0 { 0.0 } else { (x - m) / s })
        .collect()
}

/// Min-max scale to [a,b] (default [0,1] if you pass 0.0,1.0).
pub fn minmax_scale(xs: &[f64], a: f64, b: f64) -> Vec<f64> {
    if xs.is_empty() {
        return vec![];
    }
    let lo = min(xs);
    let hi = max(xs);
    if (hi - lo).abs() < f64::EPSILON {
        return xs.iter().map(|_| (a + b) / 2.0).collect();
    }
    xs.iter()
        .map(|&x| a + (x - lo) * (b - a) / (hi - lo))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approx; // macro from utils.rs
    use crate::stats::utils::EPS;

    #[test]
    fn robust_shape_and_scaling() {
        let xs2 = vec![1.0, 2.0, 3.0, 4.0];

        // z-scores: ends are ±1.161895...
        let z = zscores(&xs2);
        approx!(z[0], -1.161895003862225, EPS);
        approx!(z[3], 1.161895003862225, EPS);

        // Min-max scaling [0, 1]
        let mm = minmax_scale(&xs2, 0.0, 1.0);
        approx!(mm[0], 0.0, EPS);
        approx!(mm[1], 1.0 / 3.0, EPS);
        approx!(mm[2], 2.0 / 3.0, EPS);
        approx!(mm[3], 1.0, EPS);
    }
}

#[cfg(test)]
mod more_tests {
    use super::*;
    use crate::approx;
    use crate::stats::utils::EPS_TIGHT;

    #[test]
    fn empty_inputs_return_empty() {
        let z: Vec<f64> = zscores(&[]);
        let mm: Vec<f64> = minmax_scale(&[], 0.0, 1.0);
        assert!(z.is_empty());
        assert!(mm.is_empty());
    }

    #[test]
    fn constant_vector_behavior() {
        let xs = vec![3.0, 3.0, 3.0];
        // zscores → zeros
        let z = zscores(&xs);
        assert!(z.iter().all(|&v| v.abs() <= EPS_TIGHT));
        // minmax → midpoint of [a,b]
        let mm = minmax_scale(&xs, 0.0, 1.0);
        assert!(mm.iter().all(|&v| (v - 0.5).abs() <= EPS_TIGHT));

        let mm2 = minmax_scale(&xs, -1.0, 1.0);
        assert!(mm2.iter().all(|&v| v.abs() <= EPS_TIGHT));
    }

    #[test]
    fn zscores_mean_zero_std_one() {
        // Nontrivial spread
        let xs = vec![1.0, 2.0, 4.0, 8.0];
        let z = zscores(&xs);
        // mean ≈ 0
        let mz = z.iter().copied().sum::<f64>() / z.len() as f64;
        approx!(mz, 0.0, 1e-12);
        // sample std ≈ 1
        let vz = {
            let m = mz;
            let s: f64 = z.iter().map(|&v| (v - m) * (v - m)).sum();
            s / (z.len() as f64 - 1.0)
        };
        approx!(vz.sqrt(), 1.0, 1e-12);
    }

    #[test]
    fn minmax_to_custom_ranges() {
        let xs = vec![-2.0, 0.0, 2.0];

        // [0,1]
        let mm01 = minmax_scale(&xs, 0.0, 1.0);
        approx!(mm01[0], 0.0, EPS_TIGHT);
        approx!(mm01[1], 0.5, EPS_TIGHT);
        approx!(mm01[2], 1.0, EPS_TIGHT);

        // [-1, 1]
        let mm11 = minmax_scale(&xs, -1.0, 1.0);
        approx!(mm11[0], -1.0, EPS_TIGHT);
        approx!(mm11[1], 0.0, EPS_TIGHT);
        approx!(mm11[2], 1.0, EPS_TIGHT);

        // reversed [1,0]
        let mm10 = minmax_scale(&xs, 1.0, 0.0);
        approx!(mm10[0], 1.0, EPS_TIGHT);
        approx!(mm10[1], 0.5, EPS_TIGHT);
        approx!(mm10[2], 0.0, EPS_TIGHT);
    }

    #[test]
    fn minmax_non_uniform_spacing() {
        // Ensure linear mapping, not rank-based
        let xs = vec![0.0, 1.0, 10.0];
        let mm = minmax_scale(&xs, 0.0, 1.0);
        approx!(mm[0], 0.0, EPS_TIGHT);
        approx!(mm[1], 1.0 / 10.0, EPS_TIGHT);
        approx!(mm[2], 1.0, EPS_TIGHT);
    }
}
