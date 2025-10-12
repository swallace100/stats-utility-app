/// Welford's online algorithm.
#[derive(Clone, Copy, Debug)]
pub struct OnlineMeanVar {
    n: u64,
    mean: f64,
    m2: f64,
}
impl Default for OnlineMeanVar {
    fn default() -> Self {
        Self {
            n: 0,
            mean: 0.0,
            m2: 0.0,
        }
    }
}
impl OnlineMeanVar {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push(&mut self, x: f64) {
        self.n += 1;
        let n = self.n as f64;
        let delta = x - self.mean;
        self.mean += delta / n;
        let delta2 = x - self.mean;
        self.m2 += delta * delta2;
    }
    pub fn count(&self) -> u64 {
        self.n
    }
    pub fn mean(&self) -> f64 {
        self.mean
    }
    pub fn sample_variance(&self) -> f64 {
        if self.n < 2 {
            f64::NAN
        } else {
            self.m2 / (self.n as f64 - 1.0)
        }
    }
    pub fn sample_std(&self) -> f64 {
        self.sample_variance().sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approx;
    use crate::stats::utils::EPS_TIGHT;

    #[test]
    fn empty_state_nan_variance_and_std() {
        let omv = OnlineMeanVar::new();
        assert_eq!(omv.count(), 0);
        // mean starts at 0.0 by design
        approx!(omv.mean(), 0.0, EPS_TIGHT);
        assert!(omv.sample_variance().is_nan());
        assert!(omv.sample_std().is_nan());
    }

    #[test]
    fn single_value_mean_ok_variance_nan() {
        let mut omv = OnlineMeanVar::new();
        omv.push(42.0);
        assert_eq!(omv.count(), 1);
        approx!(omv.mean(), 42.0, EPS_TIGHT);
        assert!(omv.sample_variance().is_nan());
        assert!(omv.sample_std().is_nan());
    }

    #[test]
    fn sequence_matches_batch_stats_small() {
        // Known numbers: mean = 2.5, sample var = 10/6 = 1.666..., std ≈ 1.29099
        let xs = [1.0, 2.0, 3.0, 4.0];

        let mut omv = OnlineMeanVar::new();
        for &x in &xs {
            omv.push(x);
        }

        assert_eq!(omv.count(), 4);
        approx!(omv.mean(), 2.5, EPS_TIGHT);
        approx!(omv.sample_variance(), 1.6666666666666667, EPS_TIGHT);
        approx!(omv.sample_std(), 1.2909944487358056, EPS_TIGHT);
    }

    #[test]
    fn order_invariance_within_fp_tolerance() {
        // Welford should be stable; different order yields nearly identical results.
        let xs = [1.0, 2.0, 3.0, 4.0, 1000.0, -999.0, 0.25, 0.5, 0.75];
        let mut a = OnlineMeanVar::new();
        for &x in &xs {
            a.push(x);
        }

        let mut b = OnlineMeanVar::new();
        for &x in xs.iter().rev() {
            b.push(x);
        }

        // Looser EPS because order can affect last-bit rounding on big ranges.
        approx!(a.mean(), b.mean(), 1e-12);
        approx!(a.sample_variance(), b.sample_variance(), 1e-10);
        approx!(a.sample_std(), b.sample_std(), 1e-10);
    }

    #[test]
    fn streaming_in_chunks_equals_all_at_once() {
        let xs = [1.0, 2.0, 3.0, 4.0, 5.0];

        let mut one_shot = OnlineMeanVar::new();
        for &x in &xs {
            one_shot.push(x);
        }

        let mut chunked = OnlineMeanVar::new();
        for &x in &xs[..2] {
            chunked.push(x);
        }
        for &x in &xs[2..] {
            chunked.push(x);
        }

        approx!(one_shot.mean(), chunked.mean(), EPS_TIGHT);
        approx!(
            one_shot.sample_variance(),
            chunked.sample_variance(),
            EPS_TIGHT
        );
        approx!(one_shot.sample_std(), chunked.sample_std(), EPS_TIGHT);
    }

    #[test]
    fn clone_is_independent() {
        let mut omv = OnlineMeanVar::new();
        for &x in &[1.0, 2.0, 3.0] {
            omv.push(x);
        }
        let cloned = omv; // Copy/Clone supported

        // mutate original only
        omv.push(4.0);

        // cloned remains at the old stats
        assert_eq!(cloned.count(), 3);
        approx!(cloned.mean(), 2.0, EPS_TIGHT);
        approx!(cloned.sample_variance(), 1.0, EPS_TIGHT);

        // original has updated stats
        assert_eq!(omv.count(), 4);
        approx!(omv.mean(), 2.5, EPS_TIGHT);
        approx!(omv.sample_variance(), 1.6666666666666667, EPS_TIGHT);
    }
}
