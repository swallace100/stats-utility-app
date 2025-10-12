/// Greedy MMR selection (cosine sim). Returns indices of chosen docs.
pub fn mmr_select(cands: &[Vec<f64>], query: &[f64], lambda: f64, k: usize) -> Vec<usize> {
    assert!((0.0..=1.0).contains(&lambda));
    let n = cands.len();
    if n == 0 || k == 0 {
        return vec![];
    }

    let mut selected = Vec::<usize>::new();
    let mut remaining: HashSet<usize> = (0..n).collect();

    let sim_q: Vec<f64> = cands.iter().map(|v| cosine_similarity(v, query)).collect();

    while selected.len() < k && !remaining.is_empty() {
        let mut best = None::<(usize, f64)>;
        for &i in &remaining {
            let max_sim_to_s = if selected.is_empty() {
                0.0
            } else {
                selected
                    .iter()
                    .map(|&j| cosine_similarity(&cands[i], &cands[j]))
                    .fold(f64::NEG_INFINITY, f64::max)
            };
            let score = lambda * sim_q[i] - (1.0 - lambda) * max_sim_to_s;
            if best.map_or(true, |(_, b)| score > b) {
                best = Some((i, score));
            }
        }
        let (choice, _) = best.unwrap();
        selected.push(choice);
        remaining.remove(&choice);
    }
    selected
}

/// Coverage = unique sources / total sources present in top-k;
/// Redundancy = avg pairwise cosine; Novelty = average (1 - max cosine to earlier picks).
pub fn coverage_novelty_redundancy(topk: &[Vec<f64>], source_ids: &[usize]) -> (f64, f64, f64) {
    assert_eq!(topk.len(), source_ids.len());
    if topk.is_empty() {
        return (f64::NAN, f64::NAN, f64::NAN);
    }

    // Coverage
    let total_sources: HashSet<usize> = source_ids.iter().copied().collect();
    let coverage = total_sources.len() as f64 / source_ids.len() as f64;

    // Redundancy
    let (mean_cos, _, _, _) = pairwise_cosine_stats(topk);
    let redundancy = mean_cos;

    // Novelty (sequential)
    let mut nov_sum = 0.0;
    for i in 0..topk.len() {
        if i == 0 {
            nov_sum += 1.0;
            continue;
        }
        let max_sim = (0..i)
            .map(|j| cosine_similarity(&topk[i], &topk[j]))
            .fold(f64::NEG_INFINITY, f64::max);
        nov_sum += 1.0 - max_sim;
    }
    let novelty = nov_sum / topk.len() as f64;

    (coverage, novelty, redundancy)
}

/// Precision@k: retrieved is a list of ids in rank order; relevant is a set (or sorted vec).
pub fn precision_at_k(retrieved: &[usize], relevant: &[usize], k: usize) -> f64 {
    if k == 0 || retrieved.is_empty() {
        return 0.0;
    }
    let k = k.min(retrieved.len());
    let rel: std::collections::HashSet<usize> = relevant.iter().copied().collect();
    let hits = retrieved
        .iter()
        .take(k)
        .filter(|id| rel.contains(id))
        .count();
    hits as f64 / k as f64
}

/// Recall@k
pub fn recall_at_k(retrieved: &[usize], relevant: &[usize], k: usize) -> f64 {
    if relevant.is_empty() {
        return f64::NAN;
    }
    let rel: std::collections::HashSet<usize> = relevant.iter().copied().collect();
    let hits = retrieved
        .iter()
        .take(k)
        .filter(|id| rel.contains(id))
        .count();
    hits as f64 / rel.len() as f64
}

/// Mean Reciprocal Rank (MRR) for a single query's ranked list.
pub fn mrr(retrieved: &[usize], relevant: &[usize]) -> f64 {
    let rel: std::collections::HashSet<usize> = relevant.iter().copied().collect();
    for (i, id) in retrieved.iter().enumerate() {
        if rel.contains(id) {
            return 1.0 / (i as f64 + 1.0);
        }
    }
    0.0
}

/// DCG@k (log2 discount), helper for nDCG.
pub fn dcg_at_k(gains: &[f64], k: usize) -> f64 {
    let k = k.min(gains.len());
    (0..k)
        .map(|i| {
            let denom = (i as f64 + 2.0).log2(); // positions start at 1
            gains[i] / denom
        })
        .sum()
}

/// nDCG@k given binary relevance via ids. For graded relevance, pass gains directly to dcg_*.
pub fn ndcg_at_k(retrieved: &[usize], relevant: &[usize], k: usize) -> f64 {
    let rel: std::collections::HashSet<usize> = relevant.iter().copied().collect();
    let gains: Vec<f64> = retrieved
        .iter()
        .map(|id| if rel.contains(id) { 1.0 } else { 0.0 })
        .collect();
    let ideal_ones = relevant.len().min(k);
    let mut ideal_gains = vec![1.0; ideal_ones];
    ideal_gains.resize(k, 0.0);
    let dcg = dcg_at_k(&gains, k);
    let idcg = dcg_at_k(&ideal_gains, k);
    if idcg == 0.0 { 0.0 } else { dcg / idcg }
}

/// Average Precision (AP) for a single ranked list.
pub fn average_precision(retrieved: &[usize], relevant: &HashSet<usize>) -> f64 {
    if relevant.is_empty() {
        return f64::NAN;
    }
    let mut hits = 0usize;
    let mut sum_prec = 0.0;
    for (i, id) in retrieved.iter().enumerate() {
        if relevant.contains(id) {
            hits += 1;
            sum_prec += hits as f64 / (i as f64 + 1.0);
        }
    }
    if hits == 0 {
        0.0
    } else {
        sum_prec / hits as f64
    }
}

/// MAP over multiple queries (same lengths for inputs).
pub fn mean_average_precision(
    retrieved_lists: &[Vec<usize>],
    relevant_sets: &[HashSet<usize>],
) -> f64 {
    assert_eq!(retrieved_lists.len(), relevant_sets.len());
    let aps: Vec<f64> = retrieved_lists
        .iter()
        .zip(relevant_sets.iter())
        .map(|(r, rel)| average_precision(r, rel))
        .collect();
    mean(&aps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approx;
    use crate::stats::utils::{EPS, EPS_TIGHT};
    use std::collections::HashSet;

    #[test]
    fn retrieval_metrics_and_mmr() {
        let retrieved = vec![3usize, 1, 2, 4, 5];
        let relevant_set: HashSet<usize> = [1usize, 2].into_iter().collect();
        let relevant_vec: Vec<usize> = relevant_set.iter().copied().collect();

        // P@K / R@K / MRR
        approx!(precision_at_k(&retrieved, &relevant_vec, 3), 2.0 / 3.0, EPS);
        approx!(recall_at_k(&retrieved, &relevant_vec, 3), 1.0, EPS);
        approx!(mrr(&retrieved, &relevant_vec), 0.5, EPS);

        // DCG / nDCG
        let gains = vec![3.0, 2.0, 1.0];
        let dcg = dcg_at_k(&gains, 3);
        let expected_dcg = 3.0 / 1.0 + 2.0 / (3.0_f64).log2() + 1.0 / 2.0;
        approx!(dcg, expected_dcg, 1e-12);

        let nd = ndcg_at_k(&retrieved[..3], &[1usize, 2], 3);
        approx!(nd, 1.0, EPS);

        // AP & MAP
        let ap = {
            let rel: HashSet<usize> = [1usize, 2].into_iter().collect();
            average_precision(&retrieved, &rel)
        };
        approx!(ap, 0.5833333333333333, 1e-12);

        let retrieved_lists = vec![retrieved.clone(), vec![1usize, 6, 2]];
        let relevant_sets = vec![
            [1usize, 2].into_iter().collect::<HashSet<_>>(),
            [2usize].into_iter().collect::<HashSet<_>>(),
        ];
        let map = mean_average_precision(&retrieved_lists, &relevant_sets);
        assert!(map > 0.5 && map < 1.0);

        // MMR
        let q = vec![1.0, 0.0];
        let cands = vec![vec![1.0, 0.0], vec![0.9, 0.1], vec![0.0, 1.0]];
        let sel = mmr_select(&cands, &q, 0.7, 2);
        assert_eq!(sel.len(), 2);
        assert!(sel.contains(&0));
        assert!(sel.contains(&2));
    }

    #[test]
    fn coverage_novelty_and_hubness() {
        let e1 = vec![1.0, 0.0];
        let e2 = vec![0.0, 1.0];
        let topk = vec![e1.clone(), e1.clone(), e2.clone()];
        let sources = vec![1usize, 1, 2];

        // Coverage / novelty / redundancy
        let (coverage, novelty, redundancy) = coverage_novelty_redundancy(&topk, &sources);
        approx!(coverage, 2.0 / 3.0, 1e-12);
        approx!(redundancy, 1.0 / 3.0, 1e-12);
        approx!(novelty, 2.0 / 3.0, 1e-12);

        // Hubness: one dominant node → high Gini
        let knn = vec![vec![1usize], vec![1usize], vec![1usize]];
        let (counts, gini) = hubness_k_occurrence(&knn, 3);
        assert_eq!(counts, vec![0, 3, 0]);
        approx!(gini, 2.0 / 3.0, 1e-12);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approx;
    use crate::stats::utils::{EPS, EPS_TIGHT};
    use std::collections::HashSet;

    #[test]
    fn retrieval_metrics_and_mmr_happy_path() {
        let retrieved = vec![3usize, 1, 2, 4, 5];
        let relevant_set: HashSet<usize> = [1usize, 2].into_iter().collect();
        let relevant_vec: Vec<usize> = relevant_set.iter().copied().collect();

        // P@K / R@K / MRR
        approx!(precision_at_k(&retrieved, &relevant_vec, 3), 2.0 / 3.0, EPS);
        approx!(recall_at_k(&retrieved, &relevant_vec, 3), 1.0, EPS);
        approx!(mrr(&retrieved, &relevant_vec), 0.5, EPS);

        // DCG / nDCG
        let gains = vec![3.0, 2.0, 1.0];
        let dcg = dcg_at_k(&gains, 3);
        let expected_dcg = 3.0 / 1.0 + 2.0 / (3.0_f64).log2() + 1.0 / 2.0;
        approx!(dcg, expected_dcg, 1e-12);

        let nd = ndcg_at_k(&retrieved[..3], &[1usize, 2], 3);
        approx!(nd, 1.0, EPS);

        // AP & MAP
        let ap = {
            let rel: HashSet<usize> = [1usize, 2].into_iter().collect();
            average_precision(&retrieved, &rel)
        };
        approx!(ap, 0.5833333333333333, 1e-12);

        let retrieved_lists = vec![retrieved.clone(), vec![1usize, 6, 2]];
        let relevant_sets = vec![
            [1usize, 2].into_iter().collect::<HashSet<_>>(),
            [2usize].into_iter().collect::<HashSet<_>>(),
        ];
        let map = mean_average_precision(&retrieved_lists, &relevant_sets);
        assert!(map > 0.5 && map < 1.0);

        // MMR greedy selection
        let q = vec![1.0, 0.0];
        let cands = vec![vec![1.0, 0.0], vec![0.9, 0.1], vec![0.0, 1.0]];
        let sel = mmr_select(&cands, &q, 0.7, 2);
        assert_eq!(sel.len(), 2);
        assert!(sel.contains(&0)); // best to query
        assert!(sel.contains(&2)); // diversified pick
    }

    // -------- Edge cases & properties --------

    #[test]
    fn precision_recall_mrr_edges() {
        let retrieved = vec![1usize, 2, 3];
        let relevant = vec![9usize, 8];

        // precision: k=0 or empty retrieved
        approx!(precision_at_k(&retrieved, &relevant, 0), 0.0, EPS_TIGHT);
        approx!(precision_at_k(&[], &relevant, 3), 0.0, EPS_TIGHT);

        // recall: empty relevant → NaN
        assert!(recall_at_k(&retrieved, &[], 3).is_nan());

        // mrr: no hits → 0
        approx!(mrr(&retrieved, &relevant), 0.0, EPS_TIGHT);
    }

    #[test]
    fn dcg_ndcg_edges() {
        // dcg truncates at k
        let gains = vec![1.0, 0.0, 0.0];
        approx!(dcg_at_k(&gains, 1), 1.0 / (1.0_f64).log2(), 1e-12); // 1 / log2(1+1) = 1
        approx!(dcg_at_k(&gains, 10), dcg_at_k(&gains, 3), 1e-12); // no extra terms

        // ndcg: no relevant → IDCG=0 → return 0
        let retrieved = vec![7usize, 8, 9];
        let nd = ndcg_at_k(&retrieved, &[], 3);
        approx!(nd, 0.0, EPS_TIGHT);
    }

    #[test]
    fn ap_and_map_edges() {
        // AP: empty relevant → NaN by design
        assert!(average_precision(&[1, 2, 3], &HashSet::new()).is_nan());

        // AP: no hits → 0
        let rel: HashSet<usize> = [9usize].into_iter().collect();
        approx!(average_precision(&[1, 2, 3], &rel), 0.0, EPS_TIGHT);

        // MAP: length mismatch panics
        let rl = vec![vec![1usize, 2, 3]];
        let rs = vec![HashSet::from([1usize]), HashSet::from([2usize])];
        let result = std::panic::catch_unwind(|| mean_average_precision(&rl, &rs));
        assert!(result.is_err());
    }

    #[test]
    fn mmr_lambda_extremes_and_edges() {
        let q = vec![1.0, 0.0];
        let cands = vec![vec![1.0, 0.0], vec![0.9, 0.1], vec![0.0, 1.0]];

        // k=0 / empty cands
        assert!(mmr_select(&[], &q, 0.5, 3).is_empty());
        assert!(mmr_select(&cands, &q, 0.5, 0).is_empty());

        // lambda=1 → pure relevance: picks highest sim to query first, then next best
        let sel_relevance = mmr_select(&cands, &q, 1.0, 2);
        assert_eq!(sel_relevance[0], 0);
        assert_eq!(sel_relevance[1], 1);

        // lambda=0 → pure diversity after first pick (first pick still by relevance)
        let sel_diverse = mmr_select(&cands, &q, 0.0, 2);
        assert_eq!(sel_diverse[0], 0); // best to query
        assert_eq!(sel_diverse[1], 2); // farthest from the first
    }

    #[test]
    fn coverage_novelty_redundancy_edges() {
        // empty
        let (c, n, r) = coverage_novelty_redundancy(&[], &[]);
        assert!(c.is_nan() && n.is_nan() && r.is_nan());

        // simple sanity (also in happy path)
        let e1 = vec![1.0, 0.0];
        let e2 = vec![0.0, 1.0];
        let topk = vec![e1.clone(), e1.clone(), e2.clone()];
        let sources = vec![1usize, 1, 2];

        let (coverage, novelty, redundancy) = coverage_novelty_redundancy(&topk, &sources);
        approx!(coverage, 2.0 / 3.0, 1e-12);
        approx!(redundancy, 1.0 / 3.0, 1e-12);
        approx!(novelty, 2.0 / 3.0, 1e-12);
    }
}
