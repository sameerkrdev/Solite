use super::wallet::WalletEntry;
use crate::utils::unix_ms_now;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub email: Option<String>,
    pub google_id: Option<String>,
    pub wallets: Vec<WalletEntry>,
    pub created_at: u64,
}

impl User {
    pub fn new(
        username: String,
        password_hash: String,
        email: Option<String>,
        google_id: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            username,
            password_hash,
            email,
            google_id: google_id,
            wallets: vec![],
            created_at: unix_ms_now(),
        }
    }

    pub fn owns_address(&self, address: &str) -> bool {
        self.wallets.iter().any(|w| w.address == address)
    }

    pub fn get_wallet(&self, address: &str) -> Option<&WalletEntry> {
        self.wallets.iter().find(|w| w.address == address)
    }

    pub fn add_wallet(&mut self, wallet: WalletEntry) {
        self.wallets.push(wallet);
    }
}
