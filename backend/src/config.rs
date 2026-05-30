use std::env;

use crate::error_handler::ApiError;

#[derive(Clone)]
pub struct Config {
    pub jwt_secret: String,
    pub google_client_id: String,
    pub jwt_access_expiry_secs: i64,
    pub jwt_refresh_expiry_secs: i64,
}

impl Config {
    pub fn from_env() -> Result<Self, ApiError> {
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| ApiError::Internal("JWT_SECRET env var is required".into()))?;
        let google_client_id = env::var("GOOGLE_CLIENT_ID")
            .map_err(|_| ApiError::Internal("GOOGLE_CLIENT_ID env var is required".into()))?;

        let jwt_access_expiry_secs = env::var("JWT_ACCESS_EXPIRY_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(900);

        let jwt_refresh_expiry_secs = env::var("JWT_REFRESH_EXPIRY_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(604_800);

        Ok(Self {
            jwt_secret,
            google_client_id,
            jwt_access_expiry_secs,
            jwt_refresh_expiry_secs,
        })
    }
}
