use crate::stats::prelude::*;
use std::collections::HashMap;

/// Silhouette score using cosine distance (1 - cosine_similarity). Returns mean silhouette.
pub fn silhouette_cosine(points: &[Vec<f64>], labels: &[usize]) -> f64 {
    assert_eq!(points.len(), labels.len());
    let n = points.len();
    if n < 2 {
        return f64::NAN;
    }

    // Precompute cluster membership
    let mut clusters: HashMap<usize, Vec<usize>> = HashMap::new();
    for (i, &lab) in labels.iter().enumerate() {
        clusters.entry(lab).or_default().push(i);
    }
    if clusters.len() < 2 {
        return f64::NAN;
    }

    let mut s_sum = 0.0;
    for i in 0..n {
        let lab_i = labels[i];
        let own = &clusters[&lab_i];

        // a(i): mean intra-cluster distance
        let a = if own.len() <= 1 {
            0.0
        } else {
            let mut tot = 0.0;
            for &j in own {
                if j == i {
                    continue;
                }
                tot += 1.0 - cosine_similarity(&points[i], &points[j]);
            }
            tot / (own.len() as f64 - 1.0)
        };

        // b(i): min mean distance to other clusters
        let mut b = f64::INFINITY;
        for (&lab, idxs) in &clusters {
            if lab == lab_i {
                continue;
            }
            let mut tot = 0.0;
            for &j in idxs {
                tot += 1.0 - cosine_similarity(&points[i], &points[j]);
            }
            b = b.min(tot / idxs.len() as f64);
        }

        let si = if a == b && a == 0.0 {
            0.0
        } else {
            (b - a) / a.max(b)
        };
        s_sum += si;
    }
    s_sum / n as f64
}

/// k-occurrence counts: how often each point appears in others' kNN lists.
/// `knn_indices` is vec per anchor of length k with neighbor indices.
/// Returns (occurrence_counts, gini_coefficient).
pub fn hubness_k_occurrence(knn_indices: &[Vec<usize>], n_points: usize) -> (Vec<usize>, f64) {
    let mut counts = vec![0usize; n_points];
    for nbrs in knn_indices {
        for &j in nbrs {
            counts[j] += 1;
        }
    }
    let gini = {
        // Gini over counts as a hubness skew measure.
        let mut vals: Vec<f64> = counts.iter().map(|&c| c as f64).collect();
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = vals.len() as f64;
        if n == 0.0 {
            0.0
        } else {
            let sum: f64 = vals.iter().sum();
            if sum == 0.0 {
                0.0
            } else {
                // G = (2*Σ(i*xi))/(n*Σxi) - (n+1)/n  with i starting at 1
                let s: f64 = vals
                    .iter()
                    .enumerate()
                    .map(|(i, &x)| (i as f64 + 1.0) * x)
                    .sum();
                (2.0 * s) / (n * sum) - (n + 1.0) / n
            }
        }
    };
    (counts, gini)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approx; // approx! macro
    use crate::stats::utils::{EPS, EPS_TIGHT}; // tolerances

    // --- silhouette_cosine ---

    #[test]
    fn silhouette_two_orthogonal_clusters_is_near_one() {
        // Two tight clusters on orthogonal axes → cosine distance between clusters = 1
        let points = vec![
            vec![1.0, 0.0],
            vec![1.0, 0.0],
            vec![0.0, 1.0],
            vec![0.0, 1.0],
        ];
        let labels = vec![0usize, 0, 1, 1];
        let s = silhouette_cosine(&points, &labels);
        approx!(s, 1.0, EPS); // allow a small tolerance
    }

    #[test]
    fn silhouette_single_cluster_is_nan() {
        let points = vec![vec![1.0, 0.0], vec![1.0, 0.0]];
        let labels = vec![0usize, 0];
        let s = silhouette_cosine(&points, &labels);
        assert!(s.is_nan());
    }

    #[test]
    fn silhouette_less_than_two_points_is_nan() {
        let points = vec![vec![1.0, 0.0]];
        let labels = vec![0usize];
        let s = silhouette_cosine(&points, &labels);
        assert!(s.is_nan());
    }

    // --- hubness_k_occurrence ---

    #[test]
    fn hubness_all_point_to_one_gives_high_gini() {
        // All 3 queries pick the same neighbor (#1) as their top-1 → counts [0,3,0]
        let knn = vec![vec![1usize], vec![1usize], vec![1usize]];
        let (counts, gini) = hubness_k_occurrence(&knn, 3);
        assert_eq!(counts, vec![0, 3, 0]);
        approx!(gini, 2.0 / 3.0, 1e-12);
    }

    #[test]
    fn hubness_uniform_counts_gini_zero() {
        // Perfectly even usage across two points → counts [1,1] → gini = 0
        let knn = vec![vec![1usize], vec![0usize]];
        let (counts, gini) = hubness_k_occurrence(&knn, 2);
        assert_eq!(counts, vec![1, 1]);
        approx!(gini, 0.0, EPS_TIGHT);
    }

    #[test]
    fn hubness_empty_is_zeroed() {
        let knn: Vec<Vec<usize>> = vec![];
        let (counts, gini) = hubness_k_occurrence(&knn, 0);
        assert!(counts.is_empty());
        approx!(gini, 0.0, EPS_TIGHT);
    }
}
