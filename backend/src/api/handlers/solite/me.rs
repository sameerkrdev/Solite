use axum::{Json, extract::State};
use std::sync::Arc;

use crate::{
    api::{handlers::solite::helpers::wallet_metadata, middleware::auth::AuthUser, router::AppState},
    error_handler::ApiError,
};

pub async fn me(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = state
        .users
        .get_by_id(&auth.user_id)
        .await
        .map_err(ApiError::from)?;

    let wallets: Vec<_> = user.wallets.iter().map(wallet_metadata).collect();

    Ok(Json(serde_json::json!({
        "ok": true,
        "id": user.id,
        "username": user.username,
        "email": user.email,
        "google_id": user.google_id,
        "created_at": user.created_at,
        "wallets": wallets,
    })))
}
