use axum::{Json, extract::{Query, State}};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    api::{
        handlers::solite::helpers::ensure_user_owns_wallet,
        middleware::auth::AuthUser,
        router::AppState,
    },
    error_handler::ApiError,
    solite::wallet_service,
};

#[derive(Debug, Deserialize)]
pub struct SecretQuery {
    pub address: String,
    pub wallet_password: String,
}

pub async fn recovery_phrase(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<SecretQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    ensure_user_owns_wallet(&state.users, &auth.user_id, &query.address).await?;

    let user = state.users.get_by_id(&auth.user_id).await.map_err(ApiError::from)?;
    let wallet = user
        .get_wallet(&query.address)
        .ok_or_else(|| ApiError::WalletNotFound(query.address.clone()))?;

    let phrase = wallet_service::decrypt_recovery_phrase(wallet, &query.wallet_password)?;

    Ok(Json(serde_json::json!({
        "ok": true,
        "address": query.address,
        "recovery_phrase": phrase,
    })))
}
