use std::sync::{Arc, RwLock};

use axum::Router;

use crate::{
    accounts::AccountDb,
    api::routes::{accounts, health},
};

#[derive(Clone)]
pub struct AppState {
    pub accounts: Arc<RwLock<AccountDb>>,
}

pub fn build_router() -> Router {
    let accounts_data = AccountDb::new(&[("abc".to_string(), 100)]);

    let state = Arc::new(AppState {
        accounts: Arc::new(RwLock::new(accounts_data)),
    });

    Router::new()
        .merge(health::routes())
        .merge(accounts::routes())
        .with_state(state)
}
