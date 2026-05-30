use crate::utils::unix_ms_now;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletEntry {
    pub label: String,
    pub address: String,               // base58 pubkey — links to WalletDb
    pub encrypted_private_key: String, // AES-256 encrypted with user password
    pub recovery_phrase: String,       // BIP39 12/24 words — encrypted
    pub created_at: u64,
}

impl WalletEntry {
    pub fn new(
        address: String,
        encrypted_private_key: String,
        recovery_phrase: String,
        label: String,
    ) -> Self {
        Self {
            label,
            address,
            encrypted_private_key,
            recovery_phrase,
            created_at: unix_ms_now(),
        }
    }
}
