use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::{config::Config, error_handler::ApiError};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: i64,
    iat: i64,
    token_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_expiry_secs: i64,
    refresh_expiry_secs: i64,
}

impl JwtService {
    pub fn new(config: &Config) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(config.jwt_secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(config.jwt_secret.as_bytes()),
            access_expiry_secs: config.jwt_access_expiry_secs,
            refresh_expiry_secs: config.jwt_refresh_expiry_secs,
        }
    }

    pub fn issue_tokens(&self, user_id: &str) -> Result<TokenPair, ApiError> {
        Ok(TokenPair {
            access_token: self.create_token(user_id, "access", self.access_expiry_secs)?,
            refresh_token: self.create_token(user_id, "refresh", self.refresh_expiry_secs)?,
        })
    }

    pub fn verify_access_token(&self, token: &str) -> Result<String, ApiError> {
        self.verify_token(token, "access")
    }

    pub fn verify_refresh_token(&self, token: &str) -> Result<String, ApiError> {
        self.verify_token(token, "refresh")
    }

    fn create_token(&self, user_id: &str, token_type: &str, expiry_secs: i64) -> Result<String, ApiError> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id.to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::seconds(expiry_secs)).timestamp(),
            token_type: token_type.to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| ApiError::Internal(e.to_string()))
    }

    fn verify_token(&self, token: &str, expected_type: &str) -> Result<String, ApiError> {
        let mut validation = Validation::default();
        validation.validate_exp = true;

        let data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|_| ApiError::Unauthorized)?;

        if data.claims.token_type != expected_type {
            return Err(ApiError::Unauthorized);
        }

        Ok(data.claims.sub)
    }
}
