use crate::utils::unix_ms_now;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalletSecret {
    WithPrivateKey {
        encrypted_private_key: String, // AES-256 encrypted with user password
    },

    Withphrase {
        encrypted_recovery_phrase: String, // BIP39 12/24 words — encrypted
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletEntry {
    pub label: String,
    pub address: String, // base58 pubkey — links to WalletDb
    pub secret: WalletSecret,
    pub kdf_salt: String,
    pub nounce: String,
    pub created_at: u64,
}

impl WalletEntry {
    pub fn new(
        address: String,
        wallet_secret: WalletSecret,
        label: String,
        salt: String,
        nounce: String,
    ) -> Self {
        Self {
            label,
            address,
            secret: wallet_secret,
            kdf_salt: salt,
            nounce,
            created_at: unix_ms_now(),
        }
    }

    // func to generate the recovery phrase
    // func to generate encryption key via password or google auth pin
    // func to encrypt the recovery phrase or private key via encryption key
    // func to decrypt the encrypted recovery phrase or encrypted private key
    // func to re-generate private key via encypted recovery key
    // func to generate the key-pair vai encrypted recovery key
}
