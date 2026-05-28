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

    pub fn is_exits(&self, key: &str) -> bool {
        if !is_valid_ed25519_pubkey(key) {
            panic!("envalid pubkey")
        }

        self.wallets.get(&key.to_string()).is_some()
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

    pub fn get_balance(&self, key: &str) -> Option<u64> {
        if !is_valid_ed25519_pubkey(key) {
            panic!("envalid pubkey")
        }

        self.wallets.get(key).copied()
    }

    pub fn credit(&mut self, key: &str, amount: u64) {
        if !is_valid_ed25519_pubkey(key) {
            panic!("envalid credit pubkey")
        }

        *self.wallets.entry(key.to_string()).or_insert(0) += amount;
    }

    pub fn debit(&mut self, key: &str, amount: u64) {
        if !self.is_exits(key) {
            panic!("Account not found")
        }

        let bal = self.wallets.get_mut(key).expect("Account not found");

        if *bal < amount {
            panic!("Insufficient balance")
        }

        *bal -= amount
    }

    pub fn debit_from_system(&mut self, key: &str, amount: u64) {
        let bal = self.wallets.get_mut(key).expect("Account not found");

        if *bal < amount {
            panic!("Insufficient balance")
        }

        *bal -= amount
    }
}
