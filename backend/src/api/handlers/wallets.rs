use std::sync::Arc;

use axum::{Json, extract::State};
use serde::Serialize;
use uuid::Uuid;

use crate::api::router::AppState;

#[derive(Debug, Serialize)]
pub struct Wallet {
    id: String,
    balance: u64,
}

pub async fn create_get(State(state): State<Arc<AppState>>) -> Json<Wallet> {
    let key = Uuid::new_v4().to_string();

    let mut db = state.wallets.write().unwrap();
    let (id, balance) = db.ensure_wallet(&key);

    Json(Wallet { id, balance })
}

// pub fn list_wallets() {}
