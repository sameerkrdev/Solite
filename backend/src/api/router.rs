use std::sync::Arc;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    api::routes::{auth, health, user},
    config::Config,
    solite::{
        auth::{google::GoogleAuth, jwt::JwtService},
        db::store,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub users: store::UserDb,
    pub jwt: JwtService,
    pub google: GoogleAuth,
}

pub fn build_router() -> Router {
    let config = Config::from_env().expect("failed to load config");
    let jwt = JwtService::new(&config);
    let google = GoogleAuth::new(&config);

    let state = Arc::new(AppState {
        users: store::UserDb::new(),
        jwt,
        google,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api/v1/health", health::routes())
        .nest("/api/v1/auth", auth::routes())
        .nest("/api/v1/user", user::routes())
        .with_state(state)
        .layer(cors)
}
