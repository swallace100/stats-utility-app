/// Entropy in bits. p must be a prob. vector (sum≈1, all >=0).
pub fn entropy_bits(p: &[f64]) -> f64 {
    let eps = 1e-15;
    p.iter()
        .filter(|&&pi| pi > 0.0)
        .map(|&pi| {
            let q = (pi + eps).clamp(eps, 1.0);
            -q * q.log2()
        })
        .sum()
}

/// KL divergence D_KL(p||q) in bits. p,q prob. vectors (same length).
pub fn kl_divergence_bits(p: &[f64], q: &[f64]) -> f64 {
    assert_eq!(p.len(), q.len());
    let eps = 1e-15;

    p.iter()
        .copied() // items are (owned) f64 now
        .zip(q.iter().copied()) // (pi, qi)
        .filter(|t| t.0 > 0.0) // avoid ref patterns; read the tuple field
        .map(|(pi, qi)| {
            let pi = (pi + eps).clamp(eps, 1.0);
            let qi = (qi + eps).clamp(eps, 1.0);
            pi * (pi / qi).log2()
        })
        .sum()
}

/// Jensen–Shannon divergence in bits (symmetric, bounded \[0,1\]).
pub fn js_divergence_bits(p: &[f64], q: &[f64]) -> f64 {
    assert_eq!(p.len(), q.len());
    let m: Vec<f64> = p.iter().zip(q).map(|(&a, &b)| 0.5 * (a + b)).collect();
    0.5 * kl_divergence_bits(p, &m) + 0.5 * kl_divergence_bits(q, &m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approx;
    use crate::stats::utils::EPS;

    #[test]
    fn information_theory() {
        // Entropy of fair coin = 1 bit
        let p = vec![0.5, 0.5];
        approx!(entropy_bits(&p), 1.0, EPS);

        // KL divergence identical distributions = 0
        let q = vec![0.5, 0.5];
        approx!(kl_divergence_bits(&p, &q), 0.0, EPS);

        // JS divergence between opposite distributions = 1 bit (max for 2 classes)
        let p2 = vec![1.0, 0.0];
        let q2 = vec![0.0, 1.0];
        approx!(js_divergence_bits(&p2, &q2), 1.0, EPS);
    }
}

#[cfg(test)]
mod more_info_tests {
    use super::*;
    use crate::approx;
    use crate::stats::utils::EPS_TIGHT;

    #[test]
    fn entropy_edges_and_uniform_three() {
        // Degenerate distribution → H = 0
        let p0 = vec![1.0, 0.0, 0.0];
        approx!(entropy_bits(&p0), 0.0, EPS_TIGHT);

        // Uniform over 3 → H = log2(3)
        let p3 = vec![1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];
        approx!(entropy_bits(&p3), (3.0_f64).log2(), 1e-12);
    }

    // --- KL divergence ---

    #[test]
    #[should_panic]
    fn kl_len_mismatch_panics() {
        let _ = kl_divergence_bits(&[0.5, 0.5], &[1.0]);
    }

    #[test]
    fn kl_identical_is_zero() {
        let p = vec![0.2, 0.3, 0.5];
        approx!(kl_divergence_bits(&p, &p), 0.0, EPS_TIGHT);
    }

    #[test]
    fn kl_known_value() {
        // D_KL([0.5,0.5] || [0.9,0.1]) ≈ 0.7369655941662061 bits
        let p = vec![0.5, 0.5];
        let q = vec![0.9, 0.1];
        approx!(kl_divergence_bits(&p, &q), 0.7369655941662061, 1e-12);
    }

    #[test]
    fn kl_q_has_zero_where_p_positive_is_finite_and_large() {
        // With eps clamp, this stays finite but should be very large and > 0
        let p = vec![1.0, 0.0];
        let q = vec![0.0, 1.0];
        let d = kl_divergence_bits(&p, &q);
        assert!(d.is_finite());
        assert!(d > 10.0);
    }

    // --- JS divergence ---

    #[test]
    #[should_panic]
    fn js_len_mismatch_panics() {
        let _ = js_divergence_bits(&[1.0, 0.0], &[0.5, 0.5, 0.0]);
    }

    #[test]
    fn js_symmetry_and_zero_on_identical() {
        let p = vec![0.2, 0.3, 0.5];
        let q = vec![0.5, 0.3, 0.2];
        approx!(js_divergence_bits(&p, &p), 0.0, EPS_TIGHT);
        let d_pq = js_divergence_bits(&p, &q);
        let d_qp = js_divergence_bits(&q, &p);
        approx!(d_pq, d_qp, EPS_TIGHT); // symmetry
    }

    #[test]
    fn js_opposite_two_class_is_one_bit() {
        let p = vec![1.0, 0.0];
        let q = vec![0.0, 1.0];
        approx!(js_divergence_bits(&p, &q), 1.0, EPS_TIGHT);
    }
}
