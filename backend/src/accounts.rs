use std::collections::HashMap;

#[derive(Clone)]
pub struct AccountDb {
    pub accounts: HashMap<String, u64>,
}

impl AccountDb {
    pub fn new(initial: &[(String, u64)]) -> Self {
        let initial_accounts = initial.iter().cloned().collect();

        Self {
            accounts: initial_accounts,
        }
    }

    pub fn ensure_account(&mut self, key: &str) -> (String, u64) {
        let default_balance = 0;

        self.accounts
            .entry(key.to_string())
            .or_insert(default_balance);

        (key.to_string(), default_balance)
    }

    pub fn all_balances(&self) -> HashMap<String, u64> {
        self.accounts.clone()
    }
}
