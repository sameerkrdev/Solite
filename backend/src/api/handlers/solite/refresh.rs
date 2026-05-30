use axum::{Json, extract::State};
use serde::Deserialize;
use std::sync::Arc;

use crate::{api::router::AppState, error_handler::ApiError};

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = state.jwt.verify_refresh_token(&body.refresh_token)?;
    let tokens = state.jwt.issue_tokens(&user_id)?;

    Ok(Json(serde_json::json!({
        "ok": true,
        "user_id": user_id,
        "access_token": tokens.access_token,
        "refresh_token": tokens.refresh_token,
    })))
}
