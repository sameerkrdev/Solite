use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};

use crate::{api::router::AppState, error_handler::ApiError};

pub struct AuthUser {
    pub user_id: String,
}

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(ApiError::Unauthorized)?;

        let user_id = state.jwt.verify_access_token(token)?;
        Ok(AuthUser { user_id })
    }
}
