use axum::{Json, extract::State};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    api::{
        handlers::solite::helpers::{auth_response, build_wallet_from_request},
        router::AppState,
    },
    error_handler::ApiError,
    solite::{auth::password::hash_password, db::user::User},
};

use super::helpers::WalletCreateRequest;

#[derive(Debug, Deserialize)]
pub struct GoogleRegisterRequest {
    pub id_token: String,
    pub username: String,
    pub wallet_password: String,
    pub wallet: Option<WalletCreateRequest>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleLoginRequest {
    pub id_token: String,
}

pub async fn google_register(
    State(state): State<Arc<AppState>>,
    Json(body): Json<GoogleRegisterRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), ApiError> {
    let claims = state.google.verify_id_token(&body.id_token).await?;
    let password_hash = hash_password(&body.wallet_password)?;

    let mut built_phrase = None;
    let mut user = User::new(
        body.username,
        password_hash,
        claims.email,
        Some(claims.sub),
    );

    if let Some(wallet_req) = &body.wallet {
        let built = build_wallet_from_request(
            &state.users,
            &body.wallet_password,
            wallet_req,
        )
        .await?;
        built_phrase = built.recovery_phrase;
        user.add_wallet(built.entry);
    }

    state.users.insert(user.clone()).await?;

    let tokens = state.jwt.issue_tokens(&user.id)?;
    Ok(auth_response(&user.id, &tokens, built_phrase))
}

pub async fn google_login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<GoogleLoginRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let claims = state.google.verify_id_token(&body.id_token).await?;

    let user = state
        .users
        .get_by_google(&claims.sub)
        .await
        .map_err(|_| ApiError::Unauthorized)?;

    let tokens = state.jwt.issue_tokens(&user.id)?;

    Ok(Json(serde_json::json!({
        "ok": true,
        "user_id": user.id,
        "access_token": tokens.access_token,
        "refresh_token": tokens.refresh_token,
    })))
}
