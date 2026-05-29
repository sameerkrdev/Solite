use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{api::router::AppState, error_handler::ApiError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Wallet {
    id: String,
    balance: u64,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterWalletRequest {
    pubkey: String,
}
pub async fn register_wallet(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterWalletRequest>,
) -> Result<Json<Wallet>, ApiError> {
    let key = body.pubkey;

    let mut db = state
        .wallets
        .write()
        .map_err(|_| ApiError::Internal("wallet db lock poisoned".to_string()))?;
    let (id, balance) = db.ensure_wallet(&key)?;

    Ok(Json(Wallet { id, balance }))
}

pub async fn get_wallet(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> Result<Json<Wallet>, ApiError> {
    let wallet_db = state
        .wallets
        .read()
        .map_err(|_| ApiError::Internal("wallet db lock poisoned".to_string()))?;

    let balance = wallet_db
        .get_balance(&key)?
        .ok_or_else(|| ApiError::WalletNotFound(key.clone()))?;

    Ok(Json(Wallet { id: key, balance }))
}

pub async fn get_wallets(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Wallet>>, ApiError> {
    let wallet_db = state
        .wallets
        .read()
        .map_err(|_| ApiError::Internal("wallet db lock poisoned".to_string()))?;

    let balances = wallet_db.all_balances();

    let wallets = balances
        .into_iter()
        .map(|(id, balance)| Wallet { id, balance })
        .collect();

    Ok(Json(wallets))
}

#[derive(Serialize, Deserialize)]
pub struct AirdropRequest {
    pubkey: String,
    amount: u64,
}
pub async fn airdrop(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AirdropRequest>,
) -> Result<StatusCode, ApiError> {
    let mut wallet_db = state
        .wallets
        .write()
        .map_err(|_| ApiError::Internal("wallet db lock poisoned".to_string()))?;

    let AirdropRequest { pubkey, amount } = body;

    if amount > 50 {
        return Err(ApiError::AirdropLimitExceeded { limit: 50 });
    }

    wallet_db.credit(&pubkey, amount)?;

    Ok(StatusCode::NO_CONTENT)
}
