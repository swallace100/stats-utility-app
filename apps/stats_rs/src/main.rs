use stats_rs::{build_app, state::AppState};
use std::{env, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::{info, warn};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logging (override with: RUST_LOG="stats_rs=debug,axum=info,tower_http=info,hyper=warn")
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,axum=info,tower_http=info,hyper=warn"));
    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();

    // Load .env if present
    let _ = dotenvy::dotenv();

    // Config
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(9000);
    let addr: SocketAddr = format!("{host}:{port}").parse()?;

    // App state + router
    let state = Arc::new(AppState::default());
    let app = build_app(state);

    // Feature flags (compile-time)
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

    info!(
        "stats_rs v{} listening on {} (features: {})",
        env!("CARGO_PKG_VERSION"),
        addr,
        features
    );

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("server shut down cleanly");
    Ok(())
}

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
