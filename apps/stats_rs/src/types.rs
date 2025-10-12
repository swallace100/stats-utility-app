use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ---- Describe (JSON and CSV) ----

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DescribeInput(#[schemars(description = "Array of numbers to summarize")] pub Vec<f64>);

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DescribeOutput {
    /// Number of observations
    pub count: usize,
    /// Arithmetic mean
    pub mean: f64,
    /// Median (50th percentile)
    pub median: f64,
    /// Sample standard deviation (n-1). Returns 0.0 if count < 2
    pub std_dev: f64,
}

/// ---- /api/v1/stats/summary ----

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SummaryIn {
    /// Array of numeric values (NaN/Inf ignored server-side)
    pub values: Vec<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SummaryOut {
    /// Number of usable observations
    pub count: usize,
    pub mean: Option<f64>,
    pub median: Option<f64>,
    pub std: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub iqr: Option<f64>,
    pub mad: Option<f64>,
}

/// ---- /api/v1/stats/distribution ----

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DistIn {
    /// Array of numeric values
    pub values: Vec<f64>,
    /// Optional number of bins (>=2). If omitted, server decides.
    #[serde(default)]
    pub bins: Option<usize>,
    /// Optional quantiles to compute (0..1)
    #[serde(default)]
    pub quantiles: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DistOut {
    /// Histogram counts (length k)
    pub counts: Vec<usize>,
    /// Histogram edges (length k+1)
    pub edges: Vec<f64>,
    /// Requested quantiles as (p, value)
    pub quantiles: Vec<(f64, f64)>,
    /// Skewness (None if undefined)
    pub skewness: Option<f64>,
    /// Excess kurtosis (None if undefined)
    pub excess_kurtosis: Option<f64>,
    /// Shannon entropy in bits (None if undefined)
    pub entropy_bits: Option<f64>,
}

/// ---- /api/v1/stats/pairwise ----

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PairIn {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PairOut {
    pub covariance: Option<f64>,
    pub pearson: Option<f64>,
    pub spearman: Option<f64>,
    pub kendall: Option<f64>,
}

/// ---- Consistent error body (optional) ----

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EcdfIn {
    pub values: Vec<f64>,
    #[serde(default)]
    pub max_points: Option<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EcdfOut {
    pub xs: Vec<f64>,
    pub ps: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QqIn {
    pub values: Vec<f64>,
    #[serde(default)]
    pub robust: Option<bool>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QqOut {
    pub sample_quantiles: Vec<f64>,
    pub theoretical_quantiles: Vec<f64>,
    pub mu_hat: f64,
    pub sigma_hat: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CorrMethod {
    Pearson,
    Spearman,
    Kendall,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CorrMatrixIn {
    pub series: Vec<Vec<f64>>,
    #[serde(default)]
    pub names: Option<Vec<String>>,
    #[serde(default)]
    pub method: Option<CorrMethod>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CorrMatrixOut {
    pub size: usize,
    #[serde(default)]
    pub names: Option<Vec<String>>,
    pub matrix: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OutlierMethod {
    Zscore,
    Iqr,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OutliersIn {
    pub values: Vec<f64>,
    #[serde(default)]
    pub method: Option<OutlierMethod>,
    #[serde(default)]
    pub threshold: Option<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OutliersOut {
    pub indices: Vec<usize>,
    pub values: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NormMethod {
    Zscore,
    Minmax,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NormalizeIn {
    pub values: Vec<f64>,
    #[serde(default)]
    pub method: Option<NormMethod>,
    #[serde(default)]
    pub range: Option<(f64, f64)>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NormalizeOut {
    pub values: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BinRuleIn {
    pub values: Vec<f64>,
    #[serde(default)]
    pub rule: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BinRuleOut {
    pub bins: usize,
}
