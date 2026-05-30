use std::sync::Arc;
use std::time::{Duration, Instant};

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::{config::Config, error_handler::ApiError};

#[derive(Debug, Clone)]
pub struct GoogleClaims {
    pub sub: String,
    pub email: Option<String>,
    pub email_verified: bool,
}

#[derive(Debug, Deserialize)]
struct GoogleIdTokenClaims {
    sub: String,
    email: Option<String>,
    email_verified: Option<bool>,
    aud: String,
    iss: String,
}

#[derive(Debug, Deserialize)]
struct GoogleCertsResponse {
    keys: Vec<GoogleJwk>,
}

#[derive(Debug, Clone, Deserialize)]
struct GoogleJwk {
    kid: String,
    n: String,
    e: String,
    kty: String,
    alg: String,
}

struct CachedJwks {
    keys: Vec<GoogleJwk>,
    fetched_at: Instant,
}

#[derive(Clone)]
pub struct GoogleAuth {
    client_id: String,
    http: reqwest::Client,
    jwks: Arc<RwLock<Option<CachedJwks>>>,
}

impl GoogleAuth {
    pub fn new(config: &Config) -> Self {
        Self {
            client_id: config.google_client_id.clone(),
            http: reqwest::Client::new(),
            jwks: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn verify_id_token(&self, id_token: &str) -> Result<GoogleClaims, ApiError> {
        let header = decode_header(id_token).map_err(|_| ApiError::Unauthorized)?;
        let kid = header.kid.ok_or(ApiError::Unauthorized)?;

        let jwks = self.get_jwks().await?;
        let jwk = jwks
            .keys
            .iter()
            .find(|k| k.kid == kid)
            .ok_or(ApiError::Unauthorized)?;

        if jwk.kty != "RSA" {
            return Err(ApiError::Unauthorized);
        }

        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
            .map_err(|_| ApiError::Unauthorized)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[self.client_id.as_str()]);
        validation.set_issuer(&["accounts.google.com", "https://accounts.google.com"]);

        let token_data = decode::<GoogleIdTokenClaims>(id_token, &decoding_key, &validation)
            .map_err(|_| ApiError::Unauthorized)?;

        Ok(GoogleClaims {
            sub: token_data.claims.sub,
            email: token_data.claims.email,
            email_verified: token_data.claims.email_verified.unwrap_or(false),
        })
    }

    async fn get_jwks(&self) -> Result<GoogleCertsResponse, ApiError> {
        {
            let cache = self.jwks.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.fetched_at.elapsed() < Duration::from_secs(3600) {
                    return Ok(GoogleCertsResponse {
                        keys: cached.keys.clone(),
                    });
                }
            }
        }

        let response = self
            .http
            .get("https://www.googleapis.com/oauth2/v3/certs")
            .send()
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .error_for_status()
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .json::<GoogleCertsResponse>()
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        let mut cache = self.jwks.write().await;
        *cache = Some(CachedJwks {
            keys: response.keys.clone(),
            fetched_at: Instant::now(),
        });

        Ok(response)
    }
}
