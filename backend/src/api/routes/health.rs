use std::sync::Arc;

use axum::{Router, routing::get};

use crate::api::{handlers::health, router::AppState};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/health", get(health::health_check))
}
