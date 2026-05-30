use axum::{Json, extract::State};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    api::router::AppState,
    error_handler::ApiError,
    solite::auth::password::compare_hash_password,
};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = state
        .users
        .get_by_username(&body.username)
        .await
        .map_err(|_| ApiError::Unauthorized)?;

    let valid = compare_hash_password(&user.password, &body.password)?;
    if !valid {
        return Err(ApiError::Unauthorized);
    }

    let tokens = state.jwt.issue_tokens(&user.id)?;

    Ok(Json(serde_json::json!({
        "ok": true,
        "user_id": user.id,
        "access_token": tokens.access_token,
        "refresh_token": tokens.refresh_token,
    })))
}
