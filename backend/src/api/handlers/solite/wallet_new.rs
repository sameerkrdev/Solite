use axum::{Json, extract::State};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    api::{middleware::auth::AuthUser, router::AppState},
    error_handler::ApiError,
    solite::wallet_service,
};

#[derive(Debug, Deserialize)]
pub struct WalletNewRequest {
    pub wallet_password: String,
    pub label: String,
}

pub async fn wallet_new(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(body): Json<WalletNewRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (entry, phrase) =
        wallet_service::create_generated_wallet(&body.wallet_password, &body.label)?;

    state.users.add_wallet(&auth.user_id, entry.clone()).await?;

    Ok(Json(serde_json::json!({
        "ok": true,
        "wallet": {
            "label": entry.label,
            "address": entry.address,
            "created_at": entry.created_at,
        },
        "recovery_phrase": phrase,
    })))
}
