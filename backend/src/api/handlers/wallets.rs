use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use serde_json::from_value;

use crate::api::router::AppState;

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
) -> Json<Wallet> {
    let key = body.pubkey;

    let mut db = state.wallets.write().unwrap();
    let (id, balance) = db.ensure_wallet(&key);

    Json(Wallet { id, balance })
}

pub async fn get_wallet(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> Json<Wallet> {
    let wallet_db = state.wallets.read().unwrap();

    let balance = match wallet_db.get_balance(&key) {
        Some(val) => val,
        None => {
            panic!("Wallet not found")
        }
    };

    let value = serde_json::to_value(balance).unwrap();

    let wallet: Wallet = from_value(value).unwrap();

    Json(wallet)
}

pub async fn get_wallets(State(state): State<Arc<AppState>>) -> Json<Vec<Wallet>> {
    let wallet_db = state.wallets.read().unwrap();

    let balances = wallet_db.all_balances();

    let value = serde_json::to_value(balances).unwrap();

    let wallets: Vec<Wallet> = from_value(value).unwrap();

    Json(wallets)
}

#[derive(Serialize, Deserialize)]
pub struct AirdropRequest {
    pubkey: String,
    amount: u64,
}
pub async fn airdrop(State(state): State<Arc<AppState>>, Json(body): Json<AirdropRequest>) {
    let mut wallet_db = state.wallets.write().unwrap();

    let AirdropRequest { pubkey, amount } = body;

    if amount > 50 {
        panic!("Airdrop amount > 50")
    }

    wallet_db.credit(&pubkey, amount);
    wallet_db.debit_from_system(&pubkey, amount);
}
