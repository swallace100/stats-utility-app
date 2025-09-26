# Rust Microservice

## Responsibilities

- Robust CSV loader (headers, missing values, locale commas).
- Column typing (numeric/ordinal/categorical).
- Numeric kernels: descriptives, t-tests, chi-square, ANOVA, regression.
- Deterministic JSON outputs.

## Example Function Signatures

```rust
pub fn describe(x: &[f64]) -> DescribeOut { /* mean, sd, se, ci95, ... */ }
pub fn ttest_welch(x: &[f64], y: &[f64]) -> TTestOut { /* t, df, p, ci */ }
pub fn chisq_independence(a: &Array2<u64>) -> ChiSqOut { /* X2, df, p */ }
pub fn ols_simple(x: &[f64], y: &[f64]) -> OlsOut { /* beta0, beta1, ... */ }
```

## Commands

```bash
cargo run
curl -fsS http://localhost:9000/health
```
