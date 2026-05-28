use std::sync::Arc;

use crate::api::{handlers::wallets, router::AppState};
use axum::{Router, routing::get};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/create", get(wallets::create_get))
}
