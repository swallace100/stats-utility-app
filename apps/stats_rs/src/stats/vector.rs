use crate::stats::prelude::*;

pub fn dot(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

pub fn l2_norm(a: &[f64]) -> f64 {
    dot(a, a).sqrt()
}
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let na = l2_norm(a);
    let nb = l2_norm(b);
    if na == 0.0 || nb == 0.0 {
        return f64::NAN;
    }
    dot(a, b) / (na * nb)
}
/// Mean vector (centroid) across rows; expects non-empty list of equal-length vectors.
pub fn centroid(points: &[Vec<f64>]) -> Vec<f64> {
    let n = points.len();
    if n == 0 {
        return vec![];
    }
    let d = points[0].len();
    let mut c = vec![0.0; d];
    for p in points {
        assert_eq!(p.len(), d);
        for (i, &v) in p.iter().enumerate() {
            c[i] += v;
        }
    }
    for v in &mut c {
        *v /= n as f64;
    }
    c
}
/// Average pairwise cosine similarity inside a cluster (simple cohesion proxy).
pub fn intra_cluster_cosine(points: &[Vec<f64>]) -> f64 {
    let n = points.len();
    if n < 2 {
        return f64::NAN;
    }
    let mut s = 0.0;
    let mut m = 0usize;
    for i in 0..n {
        for j in (i + 1)..n {
            s += cosine_similarity(&points[i], &points[j]);
            m += 1;
        }
    }
    s / m as f64
}

pub fn pairwise_cosine_stats(points: &[Vec<f64>]) -> (f64, f64, f64, f64) {
    let n = points.len();
    if n < 2 {
        return (f64::NAN, f64::NAN, f64::NAN, f64::NAN);
    }
    let mut vals = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            vals.push(cosine_similarity(&points[i], &points[j]));
        }
    }
    let m = mean(&vals);
    let s = sample_std_dev(&vals, m);
    let lo = vals.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    (m, lo, hi, s)
}

/// Redundancy = average pairwise cosine; Dispersion = 1 - mean cosine.
pub fn redundancy_and_dispersion(points: &[Vec<f64>]) -> (f64, f64) {
    let (mean_cos, _, _, _) = pairwise_cosine_stats(points);
    (mean_cos, 1.0 - mean_cos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approx;
    use crate::stats::utils::{EPS, EPS_TIGHT};

    #[test]
    fn vector_ops_and_clusters() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let c = vec![1.0, 0.0];

        // Basic vector ops
        approx!(dot(&a, &b), 0.0, EPS_TIGHT);
        approx!(l2_norm(&a), 1.0, EPS_TIGHT);
        approx!(cosine_similarity(&a, &c), 1.0, EPS_TIGHT);
        approx!(cosine_similarity(&a, &b), 0.0, EPS_TIGHT);

        // Pairwise cosine stats
        let pts = vec![a.clone(), b.clone(), c.clone()];
        let (mean_cos, lo, hi, std_cos) = pairwise_cosine_stats(&pts);
        approx!(mean_cos, 1.0 / 3.0, EPS_TIGHT);
        approx!(lo, 0.0, EPS_TIGHT);
        approx!(hi, 1.0, EPS_TIGHT);
        approx!(std_cos, f64::sqrt(1.0 / 3.0), EPS_TIGHT);

        // Redundancy / dispersion (local to this module)
        let (redundancy, dispersion) = redundancy_and_dispersion(&pts);
        approx!(redundancy, 1.0 / 3.0, EPS_TIGHT);
        approx!(dispersion, 2.0 / 3.0, EPS_TIGHT);

        // Centroid
        let cent = centroid(&[vec![1.0, 2.0], vec![3.0, 4.0]]);
        assert_eq!(cent, vec![2.0, 3.0]);

        // Intra-cluster cosine
        let cluster_same = vec![vec![1.0, 0.0], vec![1.0, 0.0]];
        approx!(intra_cluster_cosine(&cluster_same), 1.0, EPS_TIGHT);

        // Silhouette (two tight, orthogonal clusters → ≈ 1.0)
        let points = vec![
            vec![1.0, 0.0],
            vec![1.0, 0.0],
            vec![0.0, 1.0],
            vec![0.0, 1.0],
        ];
        let labels = vec![0usize, 0, 1, 1];
        approx!(silhouette_cosine(&points, &labels), 1.0, EPS);
    }
}

#[cfg(test)]
mod edge_tests {
    use super::*;
    use crate::approx;
    use crate::stats::utils::EPS_TIGHT;

    // --- shape / length invariants ---

    #[test]
    #[should_panic]
    fn dot_len_mismatch_panics() {
        let _ = dot(&[1.0, 2.0], &[1.0]);
    }

    #[test]
    #[should_panic]
    fn centroid_mixed_dims_panics() {
        let _ = centroid(&[vec![1.0, 2.0], vec![3.0]]); // ragged
    }

    // --- zero vectors & NaN conventions ---

    #[test]
    fn l2_norm_zero_and_cosine_with_zero_is_nan() {
        let z = vec![0.0, 0.0];
        approx!(l2_norm(&z), 0.0, EPS_TIGHT);
        // cosine(a, 0) = NaN by design
        assert!(cosine_similarity(&[1.0, 0.0], &z).is_nan());
        assert!(cosine_similarity(&z, &[0.0, 1.0]).is_nan());
    }

    // --- insufficient samples ---

    #[test]
    fn pairwise_stats_and_intra_cluster_insufficient() {
        // n < 2 → NaNs
        let one = vec![vec![1.0, 0.0]];
        let (m, lo, hi, s) = pairwise_cosine_stats(&one);
        assert!(m.is_nan() && lo.is_nan() && hi.is_nan() && s.is_nan());

        let ic = intra_cluster_cosine(&one);
        assert!(ic.is_nan());
    }

    // --- negative similarity sanity ---

    #[test]
    fn opposite_vectors_cosine_is_minus_one() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        approx!(cosine_similarity(&a, &b), -1.0, EPS_TIGHT);

        // Pairwise stats reflect a {-1, 1, 0} mix correctly
        let pts = vec![a.clone(), b.clone(), vec![0.0, 1.0]];
        let (mean_cos, lo, hi, _std) = pairwise_cosine_stats(&pts);
        // pairs: (a,b)=-1, (a,c)=0, (b,c)=0  → mean = (-1+0+0)/3 = -1/3
        approx!(mean_cos, -1.0 / 3.0, EPS_TIGHT);
        approx!(lo, -1.0, EPS_TIGHT);
        approx!(hi, 0.0, EPS_TIGHT);
    }

    // --- centroid empty ---

    #[test]
    fn centroid_empty_returns_empty() {
        let c = centroid(&[]);
        assert!(c.is_empty());
    }
}
