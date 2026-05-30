use std::net::SocketAddr;

mod api;
mod config;
mod error_handler;
mod solite;
mod utils;

use api::router;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let app = router::build_router();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind TCP listener");

    axum::serve(listener, app)
        .await
        .expect("server error");
}
