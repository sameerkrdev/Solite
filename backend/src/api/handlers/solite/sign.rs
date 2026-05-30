use axum::{Json, extract::State};
use ed25519_dalek::Signer;
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    api::{middleware::auth::AuthUser, router::AppState},
    error_handler::ApiError,
    solite::wallet_service,
};

#[derive(Debug, Deserialize)]
pub struct SignRequest {
    pub address: String,
    pub wallet_password: String,
    pub message: String,
}

pub async fn sign(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(body): Json<SignRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (user, wallet) = state
        .users
        .get_wallet_by_address(&body.address)
        .await
        .map_err(ApiError::from)?;

    if user.id != auth.user_id {
        return Err(ApiError::Forbidden);
    }

    let keypair = wallet_service::unlock_keypair(&wallet, &body.wallet_password)?;

    let message_bytes = hex::decode(&body.message)
        .map_err(|_| ApiError::MissingField("message must be hex-encoded".into()))?;

    let signing_key = ed25519_dalek::SigningKey::from_bytes(&keypair.private_key);
    let signature = signing_key.sign(&message_bytes);

    Ok(Json(serde_json::json!({
        "ok": true,
        "address": body.address,
        "signature": hex::encode(signature.to_bytes()),
    })))
}
