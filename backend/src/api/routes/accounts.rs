use std::sync::Arc;

use crate::api::{handlers::accounts, router::AppState};
use axum::{Router, routing::get};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/create", get(accounts::create_get))
}
