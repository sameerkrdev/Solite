use axum::{Json, extract::State};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    api::{
        handlers::solite::helpers::{ensure_address_available, wallet_metadata},
        middleware::auth::AuthUser,
        router::AppState,
    },
    error_handler::ApiError,
    solite::wallet_service,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportMode {
    Phrase,
    Privkey,
}

#[derive(Debug, Deserialize)]
pub struct WalletImportRequest {
    pub mode: ImportMode,
    pub wallet_password: String,
    pub label: String,
    pub phrase: Option<String>,
    pub private_key: Option<String>,
}

pub async fn wallet_import(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(body): Json<WalletImportRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let entry = match body.mode {
        ImportMode::Phrase => {
            let phrase = body
                .phrase
                .as_deref()
                .ok_or_else(|| ApiError::MissingField("phrase".into()))?;
            let entry =
                wallet_service::import_phrase_wallet(&body.wallet_password, phrase, &body.label)?;
            ensure_address_available(&state.users, &entry.address).await?;
            entry
        }
        ImportMode::Privkey => {
            let private_key = body
                .private_key
                .as_deref()
                .ok_or_else(|| ApiError::MissingField("private_key".into()))?;
            let entry = wallet_service::import_privkey_wallet(
                &body.wallet_password,
                private_key,
                &body.label,
            )?;
            ensure_address_available(&state.users, &entry.address).await?;
            entry
        }
    };

    state.users.add_wallet(&auth.user_id, entry.clone()).await?;

    Ok(Json(serde_json::json!({
        "ok": true,
        "wallet": wallet_metadata(&entry),
    })))
}
