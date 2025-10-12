use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DescribeInput(#[schemars(description = "Array of numbers to summarize")] pub Vec<f64>);

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
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
