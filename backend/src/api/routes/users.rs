use std::sync::Arc;

use crate::api::{handlers::solite, router::AppState};
use axum::{Router, routing::post};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/", post(solite::users::create_user))
}
