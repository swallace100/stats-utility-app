// src/stats/mod.rs
pub mod basic;
pub mod cluster;
pub mod corr;
pub mod drift;
pub mod info;
pub mod online;
pub mod preprocess;
#[cfg(feature = "rag")]
pub mod rag;
pub mod robust;
pub mod vector;

pub use basic::*;
pub use cluster::*;
pub use corr::*;
pub use drift::*;
pub use info::*;
pub use online::*;
pub use preprocess::*;
#[cfg(feature = "rag")]
pub use rag::*;
pub use robust::*;
pub use vector::*;

mod utils;

/// Handy prelude for routes and downstream crates.
pub mod prelude {
    pub use super::{
        OnlineMeanVar,
        average_ranks,
        centroid,
        cosine_similarity,
        // corr / shape
        covariance,
        // vector / cluster / info / drift / online
        dot,
        entropy_bits,
        excess_kurtosis,
        intra_cluster_cosine,
        iqr,
        js_divergence_bits,
        kendall_tau_b,
        kl_divergence_bits,
        l2_norm,
        mad,
        max,
        mean,
        median,
        min,
        minmax_scale,
        mode,
        pairwise_cosine_stats,
        pearson_correlation,
        population_std_dev,
        population_variance,
        psi_quantile_bins,
        quantile,
        quartiles,
        range,
        sample_std_dev,
        sample_variance,
        silhouette_cosine,
        skewness,
        spearman_rho,
        // basic
        sum,
        // preprocess
        zscores,
    };

    // Feature-gated RAG re-exports must be a separate item:
    #[cfg(feature = "rag")]
    pub use super::{
        average_precision, coverage_novelty_redundancy, dcg_at_k, mean_average_precision,
        mmr_select, mrr, ndcg_at_k, precision_at_k, recall_at_k,
    };
}
