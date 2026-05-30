use axum::{Json, extract::{Query, State}};
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

use super::recovery::SecretQuery;

pub async fn private_key(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<SecretQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    ensure_user_owns_wallet(&state.users, &auth.user_id, &query.address).await?;

    let user = state.users.get_by_id(&auth.user_id).await.map_err(ApiError::from)?;
    let wallet = user
        .get_wallet(&query.address)
        .ok_or_else(|| ApiError::WalletNotFound(query.address.clone()))?;

    let keypair = wallet_service::unlock_keypair(wallet, &query.wallet_password)?;
    let private_key = bs58::encode(keypair.private_key).into_string();

    Ok(Json(serde_json::json!({
        "ok": true,
        "address": query.address,
        "private_key": private_key,
    })))
}
