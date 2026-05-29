use std::sync::{Arc, RwLock};

use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    api::routes::{health, wallets},
    wallets::WalletDb,
};

#[derive(Clone)]
pub struct AppState {
    pub wallets: Arc<RwLock<WalletDb>>,
}

pub fn build_router() -> Router {
    let wallets_data = WalletDb::new(&[("SystemProgramWallet".to_string(), 1_000_000)]);

    let state = Arc::new(AppState {
        wallets: Arc::new(RwLock::new(wallets_data)),
    });

    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any);

    Router::new()
        .merge(health::routes())
        .merge(wallets::routes())
        .with_state(state)
        .layer(cors)
}
