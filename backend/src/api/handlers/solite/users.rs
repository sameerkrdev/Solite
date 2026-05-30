use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::{api::router::AppState, error_handler::ApiError, solite::db::user::User, utils};

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    username: String,
    password: String,
    email: Option<String>,
    google_id: Option<String>,
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let CreateUserRequest {
        username,
        password,
        email,
        google_id,
    } = body;

    let password_hash = utils::hash_password(&password)?;
    let new_user = User::new(username, password_hash, email, google_id);

    state.users.insert(new_user.clone()).await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "id": new_user.id
        })),
    ))
}
