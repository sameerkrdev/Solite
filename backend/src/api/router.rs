use std::sync::Arc;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    api::routes::{health, users},
    solite::db::store,
};

#[derive(Clone)]
pub struct AppState {
    pub users: store::UserDb,
}

pub fn build_router() -> Router {
    let users_data = store::UserDb::new();

    let state = Arc::new(AppState { users: users_data });

    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any);

    Router::new()
        .nest("/api/v1/health", health::routes())
        .nest("/api/v1/users", users::routes())
        .with_state(state)
        .layer(cors)
}
