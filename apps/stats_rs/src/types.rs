//! # Data Transfer Types for `stats_rs`
//!
//! This module defines all request and response payloads exchanged
//! between clients and the `stats_rs` microservice.
//!
//! Each struct derives [`Serialize`], [`Deserialize`], and [`JsonSchema`],
//! allowing automatic JSON (de)serialization and OpenAPI schema generation.
//!
//! The models are grouped by their corresponding endpoints:
//! - `/describe` and `/describe-csv` → [`DescribeInput`], [`DescribeOutput`]
//! - `/stats/summary` → [`SummaryIn`], [`SummaryOut`]
//! - `/stats/distribution` → [`DistIn`], [`DistOut`]
//! - `/stats/pairwise` → [`PairIn`], [`PairOut`]
//! - `/stats/ecdf` → [`EcdfIn`], [`EcdfOut`]
//! - `/stats/qq-normal` → [`QqIn`], [`QqOut`]
//! - `/stats/corr-matrix` → [`CorrMatrixIn`], [`CorrMatrixOut`]
//! - `/stats/outliers` → [`OutliersIn`], [`OutliersOut`]
//! - `/stats/normalize` → [`NormalizeIn`], [`NormalizeOut`]
//! - `/stats/binrule` → [`BinRuleIn`], [`BinRuleOut`]
//!
//! These definitions are used by both the backend (Axum routes) and
//! the frontend contracts (e.g., via `@your-scope/contracts`).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ---- `/api/v1/describe` and `/api/v1/describe-csv` ----
/// Request body for basic descriptive statistics.
///
/// Accepts a vector of numeric values (from JSON or parsed CSV column).
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DescribeInput(#[schemars(description = "Array of numbers to summarize")] pub Vec<f64>);

/// Response body containing common summary statistics.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DescribeOutput {
    /// Number of observations (`n`)
    pub count: usize,
    /// Arithmetic mean
    pub mean: f64,
    /// Median (50th percentile)
    pub median: f64,
    /// Sample standard deviation (n−1). Returns 0.0 if `count < 2`
    pub std_dev: f64,
}

/// ---- `/api/v1/stats/summary` ----
/// Input for summary statistics endpoint.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SummaryIn {
    /// Array of numeric values (NaN/Inf ignored server-side)
    pub values: Vec<f64>,
}

/// Output containing various univariate summary metrics.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SummaryOut {
    /// Number of usable observations
    pub count: usize,
    /// Arithmetic mean
    pub mean: Option<f64>,
    /// Median (50th percentile)
    pub median: Option<f64>,
    /// Sample standard deviation
    pub std: Option<f64>,
    /// Minimum value
    pub min: Option<f64>,
    /// Maximum value
    pub max: Option<f64>,
    /// Interquartile range (Q3 − Q1)
    pub iqr: Option<f64>,
    /// Median absolute deviation
    pub mad: Option<f64>,
}

/// ---- `/api/v1/stats/distribution` ----
/// Request body for histogram, quantile, and entropy computations.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DistIn {
    /// Array of numeric values
    pub values: Vec<f64>,
    /// Optional number of bins (≥2). If omitted, server decides.
    #[serde(default)]
    pub bins: Option<usize>,
    /// Optional quantiles to compute (0..1)
    #[serde(default)]
    pub quantiles: Option<Vec<f64>>,
}

/// Response body containing histogram data and shape statistics.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DistOut {
    /// Histogram counts (length *k*)
    pub counts: Vec<usize>,
    /// Histogram bin edges (length *k + 1*)
    pub edges: Vec<f64>,
    /// Requested quantiles as `(p, value)` pairs
    pub quantiles: Vec<(f64, f64)>,
    /// Skewness (None if undefined)
    pub skewness: Option<f64>,
    /// Excess kurtosis (None if undefined)
    pub excess_kurtosis: Option<f64>,
    /// Shannon entropy in bits (None if undefined)
    pub entropy_bits: Option<f64>,
}

/// ---- `/api/v1/stats/pairwise` ----
/// Input for pairwise correlation and covariance calculations.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PairIn {
    /// First numeric series
    pub x: Vec<f64>,
    /// Second numeric series
    pub y: Vec<f64>,
}

/// Output with covariance and correlation coefficients.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PairOut {
    pub covariance: Option<f64>,
    pub pearson: Option<f64>,
    pub spearman: Option<f64>,
    pub kendall: Option<f64>,
}

/// ---- Consistent error response ----
/// Standardized error response body used across all endpoints.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ErrorResponse {
    /// Short error code (e.g. `"invalid_data"`)
    pub code: String,
    /// Human-readable error message
    pub message: String,
}

/// ---- `/api/v1/stats/ecdf` ----
/// Request for empirical CDF (ECDF) calculation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EcdfIn {
    /// Input numeric series
    pub values: Vec<f64>,
    /// Optional downsampling cap for large datasets
    #[serde(default)]
    pub max_points: Option<usize>,
}

/// Response containing ECDF points (x, p(x)).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EcdfOut {
    /// Sorted sample values
    pub xs: Vec<f64>,
    /// Corresponding cumulative probabilities
    pub ps: Vec<f64>,
}

/// ---- `/api/v1/stats/qq-normal` ----
/// Input for Q–Q plot computation against a normal distribution.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QqIn {
    /// Sample values to compare against normal quantiles
    pub values: Vec<f64>,
    /// If true, use robust estimators for μ̂ and σ̂
    #[serde(default)]
    pub robust: Option<bool>,
}

/// Output with theoretical vs. sample quantiles and fit parameters.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QqOut {
    /// Empirical sample quantiles
    pub sample_quantiles: Vec<f64>,
    /// Theoretical quantiles under normality
    pub theoretical_quantiles: Vec<f64>,
    /// Estimated mean (μ̂)
    pub mu_hat: f64,
    /// Estimated standard deviation (σ̂)
    pub sigma_hat: f64,
}

/// ---- `/api/v1/stats/corr-matrix` ----
/// Available correlation methods for matrix computation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CorrMethod {
    /// Pearson correlation (linear)
    Pearson,
    /// Spearman rank correlation
    Spearman,
    /// Kendall rank correlation
    Kendall,
}

/// Input for correlation matrix endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CorrMatrixIn {
    /// List of numeric series; all must be equal length
    pub series: Vec<Vec<f64>>,
    /// Optional names for each series (for labeling output)
    #[serde(default)]
    pub names: Option<Vec<String>>,
    /// Correlation method (defaults to Pearson)
    #[serde(default)]
    pub method: Option<CorrMethod>,
}

/// Output correlation matrix in flattened (row-major) format.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CorrMatrixOut {
    /// Matrix size (n×n)
    pub size: usize,
    /// Optional variable names
    #[serde(default)]
    pub names: Option<Vec<String>>,
    /// Flattened correlation matrix (row-major order)
    pub matrix: Vec<f64>,
}

/// ---- `/api/v1/stats/outliers` ----
/// Available outlier detection methods.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OutlierMethod {
    /// Z-score thresholding
    Zscore,
    /// Interquartile range (IQR) rule
    Iqr,
}

/// Input for outlier detection.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OutliersIn {
    /// Input numeric series
    pub values: Vec<f64>,
    /// Method to use (`zscore` or `iqr`)
    #[serde(default)]
    pub method: Option<OutlierMethod>,
    /// Threshold multiplier (e.g. 3 for z-score)
    #[serde(default)]
    pub threshold: Option<f64>,
}

/// Output listing detected outliers.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OutliersOut {
    /// Indices of detected outliers
    pub indices: Vec<usize>,
    /// Values corresponding to detected outliers
    pub values: Vec<f64>,
}

/// ---- `/api/v1/stats/normalize` ----
/// Normalization methods supported by `/normalize`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NormMethod {
    /// Standard score (Z-score) normalization
    Zscore,
    /// Min–max scaling to a specified range
    Minmax,
}

/// Input for data normalization.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NormalizeIn {
    /// Input numeric series
    pub values: Vec<f64>,
    /// Method (defaults to `zscore`)
    #[serde(default)]
    pub method: Option<NormMethod>,
    /// Range for min–max normalization, e.g. (0.0, 1.0)
    #[serde(default)]
    pub range: Option<(f64, f64)>,
}

/// Output containing normalized values.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NormalizeOut {
    pub values: Vec<f64>,
}

/// ---- `/api/v1/stats/binrule` ----
/// Input specifying a binning rule for histogram selection.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BinRuleIn {
    /// Numeric series to analyze
    pub values: Vec<f64>,
    /// Optional binning rule (`sturges`, `sqrt`, `fd`, etc.)
    #[serde(default)]
    pub rule: Option<String>,
}

/// Output with computed number of histogram bins.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BinRuleOut {
    /// Number of bins chosen by rule
    pub bins: usize,
}
