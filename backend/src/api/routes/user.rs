use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::api::{handlers::solite, router::AppState};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/me", get(solite::me::me))
        .route("/wallets", get(solite::wallets::list_wallets))
        .route("/wallet/new", post(solite::wallet_new::wallet_new))
        .route("/wallet/import", post(solite::wallet_import::wallet_import))
        .route("/recovery-phrase", get(solite::recovery::recovery_phrase))
        .route("/private-key", get(solite::privkey::private_key))
        .route("/sign", post(solite::sign::sign))
}
