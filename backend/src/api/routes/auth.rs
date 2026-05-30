use std::sync::Arc;

use axum::{Router, routing::post};
use crate::api::{handlers::solite, router::AppState};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(solite::register::register))
        .route("/login", post(solite::login::login))
        .route("/google/register", post(solite::google::google_register))
        .route("/google/login", post(solite::google::google_login))
        .route("/refresh", post(solite::refresh::refresh))
}
