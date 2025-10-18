//! Route module aggregator: re-exports to preserve `routes::*` API.

pub mod describe;
pub mod docs;
pub mod health;
pub mod prom;
pub mod schemas;
pub mod stats_binrule;
pub mod stats_corr_matrix;
pub mod stats_distribution;
pub mod stats_ecdf;
pub mod stats_normalize;
pub mod stats_outliers;
pub mod stats_pairwise;
pub mod stats_qq;
pub mod stats_summary;

// Re-exports (public surface preserved)
pub use describe::{describe, describe_csv};
pub use docs::{docs_ui, swagger_ui};
pub use health::{health, ready};
pub use prom::prom_metrics;
pub use schemas::{openapi, schema_describe_input, schema_describe_output};

pub use stats_binrule::stats_binrule;
pub use stats_corr_matrix::stats_corr_matrix;
pub use stats_distribution::stats_distribution;
pub use stats_ecdf::stats_ecdf;
pub use stats_normalize::stats_normalize;
pub use stats_outliers::stats_outliers;
pub use stats_pairwise::stats_pairwise;
pub use stats_qq::stats_qq_normal;
pub use stats_summary::stats_summary;
