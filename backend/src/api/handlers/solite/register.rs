use axum::{Json, extract::State, http::StatusCode};
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
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub wallet: Option<WalletCreateRequest>,
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let password_hash = hash_password(&body.password)?;

    let mut built_phrase = None;
    let mut user = User::new(
        body.username,
        password_hash,
        body.email,
        None,
    );

    if let Some(wallet_req) = &body.wallet {
        let built = build_wallet_from_request(&state.users, &body.password, wallet_req).await?;
        built_phrase = built.recovery_phrase;
        user.add_wallet(built.entry);
    }

    state.users.insert(user.clone()).await?;

    let tokens = state.jwt.issue_tokens(&user.id)?;
    Ok(auth_response(&user.id, &tokens, built_phrase))
}
