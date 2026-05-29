use std::sync::Arc;

use crate::api::{handlers::wallets, router::AppState};
use axum::{
    Router,
    routing::{get, post},
};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(wallets::register_wallet))
        .route("/", get(wallets::get_wallets))
        .route("/{key}", get(wallets::get_wallet))
        .route("/airdrop", post(wallets::airdrop))
}
