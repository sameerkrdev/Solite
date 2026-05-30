use crate::utils::unix_ms_now;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalletSecret {
    WithPrivateKey {
        encrypted_private_key: String,
    },
    WithPhrase {
        encrypted_recovery_phrase: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletEntry {
    pub label: String,
    pub address: String,
    pub secret: WalletSecret,
    pub kdf_salt: String,
    pub created_at: u64,
}

impl WalletEntry {
    pub fn new(
        label: String,
        address: String,
        secret: WalletSecret,
        kdf_salt: String,
    ) -> Self {
        Self {
            label,
            address,
            secret,
            kdf_salt,
            created_at: unix_ms_now(),
        }
    }
}
