use std::sync::Arc;

use axum::{Json, extract::State};
use serde::Serialize;
use uuid::Uuid;

use crate::api::router::AppState;

#[derive(Debug, Serialize)]
pub struct Account {
    id: String,
    balance: u64,
}

pub async fn create_get(State(state): State<Arc<AppState>>) -> Json<Account> {
    let key = Uuid::new_v4().to_string();

    let mut db = state.accounts.write().unwrap();
    let (id, balance) = db.ensure_account(&key);

    Json(Account { id, balance })
}

// pub fn list_accounts() {}
