//! # `stats_rs` Main Entrypoint
//!
//! The executable entrypoint for the **Stats Utility App’s Rust microservice**.
//! This binary initializes logging, loads configuration, builds the Axum router,
//! and starts the HTTP server with graceful shutdown.
//!
//! ## Responsibilities
//!
//! - Initialize structured tracing via [`tracing_subscriber`]
//! - Load environment configuration (optionally from `.env`)
//! - Build the Axum router with [`build_app`] and shared [`AppState`]
//! - Report active compile-time features (`rag`, `docs`, `metrics`)
//! - Serve incoming HTTP traffic on the configured address
//! - Handle termination gracefully (SIGTERM, Ctrl+C)
//!
//! ## Environment Variables
//!
//! | Variable | Default | Description |
//! |-----------|----------|-------------|
//! | `HOST` | `0.0.0.0` | Network interface to bind |
//! | `PORT` | `9000` | TCP port for the HTTP server |
//! | `RUST_LOG` | `info,axum=info,tower_http=info,hyper=warn` | Logging filter spec |
//!
//! Example `.env` file:
//! ```env
//! HOST=0.0.0.0
//! PORT=9000
//! RUST_LOG=stats_rs=debug,axum=info
//! ```
//!
//! ## Running
//!
//! ```bash
//! cargo run --release
//! ```
//!
//! ## Graceful Shutdown
//!
//! The server listens for `SIGTERM` and `Ctrl+C` (Unix or Windows).
//! Upon receiving either signal, it stops accepting new requests,
//! waits for in-flight requests to complete, and then exits cleanly.

use stats_rs::{build_app, state::AppState};
use std::{env, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::{info, warn};
use tracing_subscriber::{EnvFilter, fmt};

/// Application entrypoint for the `stats_rs` microservice.
///
/// Initializes tracing, loads environment configuration,
/// builds the router via [`build_app`], and starts an async HTTP server.
///
/// The runtime is managed by the Tokio multi-threaded scheduler
/// (`#[tokio::main]` macro), which allows high concurrency
/// for large statistical workloads.
///
/// # Returns
///
/// Returns `anyhow::Result<()>` to propagate initialization or I/O errors.
///
/// # Panics
///
/// This function may panic if:
/// - The bound socket address cannot be parsed or opened.
/// - The tracing subscriber fails to initialize.
/// - Environment variables contain invalid UTF-8 or parse values.
///
/// # Example
///
/// ```bash
/// RUST_LOG=stats_rs=debug cargo run
/// ```
///
/// You should see:
/// ```text
/// stats_rs v1.0.0 listening on 0.0.0.0:9000 (features: none)
/// ```
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // --- Logging Setup -------------------------------------------------------
    // Default filter: info-level logs for core and framework crates.
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,axum=info,tower_http=info,hyper=warn"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();

    // --- Environment Setup ---------------------------------------------------
    // Load `.env` file if available (no error if missing)
    let _ = dotenvy::dotenv();

    // Load network configuration
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(9000);
    let addr: SocketAddr = format!("{host}:{port}").parse()?;

    // --- Application State + Router ------------------------------------------
    let state = Arc::new(AppState);
    let app = build_app(state);

    // --- Feature Flag Detection ----------------------------------------------
    // Uses compile-time flags (Cargo features) to log enabled modules.
    let features = String::new();
    #[cfg(feature = "rag")]
    {
        features.push_str("rag, ");
    }
    #[cfg(feature = "docs")]
    {
        features.push_str("docs, ");
    }
    #[cfg(feature = "metrics")]
    {
        features.push_str("metrics, ");
    }
    let features = if features.is_empty() {
        "none".to_string()
    } else {
        features.trim_end_matches([',', ' ']).to_string()
    };

    // --- Startup Log ---------------------------------------------------------
    info!(
        "stats_rs v{} listening on {} (features: {})",
        env!("CARGO_PKG_VERSION"),
        addr,
        features
    );

    // --- Server Startup ------------------------------------------------------
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("server shut down cleanly");
    Ok(())
}

/// Waits for OS signals to trigger a graceful shutdown.
///
/// The handler supports:
/// - `Ctrl+C` (SIGINT)
/// - `SIGTERM` (on Unix)
///
/// Once a signal is received, the function returns,
/// allowing [`axum::serve`] to finish active requests.
///
/// # Example
///
/// ```rust,ignore
/// axum::serve(listener, app)
///     .with_graceful_shutdown(shutdown_signal())
///     .await?;
/// ```
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut term = signal(SignalKind::terminate()).expect("install SIGTERM handler");

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = term.recv() => {},
        }
    }

    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }

    warn!("shutdown signal received");
}
