/// Very tight epsilon for deterministic numerics
#[allow(dead_code)]
pub const EPS_TIGHT: f64 = 1e-12;

/// Looser epsilon for numerics that accumulate a bit more error
#[allow(dead_code)]
pub const EPS: f64 = 1e-9;

#[cfg(test)]
#[allow(dead_code)]
#[inline]
pub fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() <= eps
}

#[cfg(test)]
#[macro_export]
macro_rules! approx {
    ($lhs:expr, $rhs:expr, $eps:expr) => {{
        let (l, r, e) = ($lhs, $rhs, $eps);
        assert!((l - r).abs() <= e, "approx failed: |{} - {}| > {}", l, r, e);
    }};
}

#[cfg(test)]
#[macro_export]
macro_rules! vec_close {
    ($lhs:expr, $rhs:expr, $eps:expr) => {{
        let (a, b) = (&$lhs, &$rhs);
        assert_eq!(a.len(), b.len(), "len mismatch: {} vs {}", a.len(), b.len());
        for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
            assert!(
                (*x - *y).abs() <= $eps,
                "idx {} -> {} vs {} (eps={})",
                i,
                x,
                y,
                $eps
            );
        }
    }};
}
