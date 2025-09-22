use stats_rs::{build_app, state::AppState};
use std::{net::SocketAddr, sync::Arc};
use tracing::info; // <-- use the lib crate

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let app_state = Arc::new(AppState::default());
    let app = build_app(app_state); // <-- from lib

    let addr: SocketAddr = ([0, 0, 0, 0], 9000).into();
    info!("stats_rs listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
