use std::collections::HashMap;

use crate::utils::is_valid_ed25519_pubkey;

#[derive(Clone)]
pub struct WalletDb {
    pub wallets: HashMap<String, u64>,
}

impl WalletDb {
    pub fn new(initial: &[(String, u64)]) -> Self {
        let initial_wallets = initial.iter().cloned().collect();

        Self {
            wallets: initial_wallets,
        }
    }

    pub fn ensure_wallet(&mut self, key: &str) -> (String, u64) {
        if !is_valid_ed25519_pubkey(key) {
            panic!("envalid pubkey")
        }

        let default_balance = 0;

        self.wallets
            .entry(key.to_string())
            .or_insert(default_balance);

        (key.to_string(), default_balance)
    }

    pub fn all_balances(&self) -> HashMap<String, u64> {
        self.wallets.clone()
    }
}
