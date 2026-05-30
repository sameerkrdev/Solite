use axum::http::StatusCode;
use serde::Deserialize;

use crate::{
    error_handler::ApiError,
    solite::{
        db::{store::UserDb, wallet::WalletEntry},
        wallet_service,
    },
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletMode {
    Generate,
    ImportPhrase,
    ImportPrivkey,
}

#[derive(Debug, Deserialize)]
pub struct WalletCreateRequest {
    pub mode: WalletMode,
    pub label: String,
    pub phrase: Option<String>,
    pub private_key: Option<String>,
}

pub struct BuiltWallet {
    pub entry: WalletEntry,
    pub recovery_phrase: Option<String>,
}

pub async fn build_wallet_from_request(
    users: &UserDb,
    password: &str,
    wallet: &WalletCreateRequest,
) -> Result<BuiltWallet, ApiError> {
    match wallet.mode {
        WalletMode::Generate => {
            let (entry, phrase) =
                wallet_service::create_generated_wallet(password, &wallet.label)?;
            Ok(BuiltWallet {
                entry,
                recovery_phrase: Some(phrase),
            })
        }
        WalletMode::ImportPhrase => {
            let phrase = wallet
                .phrase
                .as_deref()
                .ok_or_else(|| ApiError::MissingField("phrase".into()))?;

            let entry = wallet_service::import_phrase_wallet(password, phrase, &wallet.label)?;
            ensure_address_available(users, &entry.address).await?;
            Ok(BuiltWallet {
                entry,
                recovery_phrase: None,
            })
        }
        WalletMode::ImportPrivkey => {
            let private_key = wallet
                .private_key
                .as_deref()
                .ok_or_else(|| ApiError::MissingField("private_key".into()))?;

            let entry =
                wallet_service::import_privkey_wallet(password, private_key, &wallet.label)?;
            ensure_address_available(users, &entry.address).await?;
            Ok(BuiltWallet {
                entry,
                recovery_phrase: None,
            })
        }
    }
}

pub async fn ensure_address_available(users: &UserDb, address: &str) -> Result<(), ApiError> {
    if users.owner_of(address).await.is_some() {
        return Err(ApiError::Conflict(format!(
            "wallet address already exists: {address}"
        )));
    }
    Ok(())
}

pub async fn ensure_user_owns_wallet(
    users: &UserDb,
    user_id: &str,
    address: &str,
) -> Result<(), ApiError> {
    if !users.owns_address(user_id, address).await {
        return Err(ApiError::Forbidden);
    }
    Ok(())
}

pub fn wallet_metadata(entry: &WalletEntry) -> serde_json::Value {
    serde_json::json!({
        "label": entry.label,
        "address": entry.address,
        "created_at": entry.created_at,
    })
}

pub fn auth_response(
    user_id: &str,
    tokens: &crate::solite::auth::jwt::TokenPair,
    recovery_phrase: Option<String>,
) -> (StatusCode, axum::Json<serde_json::Value>) {
    let mut body = serde_json::json!({
        "ok": true,
        "user_id": user_id,
        "access_token": tokens.access_token,
        "refresh_token": tokens.refresh_token,
    });

    if let Some(phrase) = recovery_phrase {
        body["recovery_phrase"] = serde_json::Value::String(phrase);
    }

    (StatusCode::CREATED, axum::Json(body))
}
