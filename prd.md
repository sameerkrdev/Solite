# Mini Solana Validator Runtime
## Step-by-Step Developer Build Guide

> One feature at a time. Each step ends with something that compiles and is verifiable.
> Never move to the next step until the current one works.

---

## Table of Contents

- [Stage 1 — Project Skeleton](#stage-1--project-skeleton)
- [Stage 2 — Domain Types](#stage-2--domain-types)
- [Stage 3 — AccountsDB](#stage-3--accountsdb)
- [Stage 4 — PoH Chain](#stage-4--poh-chain)
- [Stage 5 — Runtime Executor](#stage-5--runtime-executor)
- [Stage 6 — Block Builder](#stage-6--block-builder)
- [Stage 7 — Mempool](#stage-7--mempool)
- [Stage 8 — Leader Scheduler](#stage-8--leader-scheduler)
- [Stage 9 — Network Bus](#stage-9--network-bus)
- [Stage 10 — Slot Clock](#stage-10--slot-clock)
- [Stage 11 — Single Validator Loop](#stage-11--single-validator-loop)
- [Stage 12 — Four Validators + Simulation](#stage-12--four-validators--simulation)
- [Stage 13 — Consensus Engine](#stage-13--consensus-engine)
- [Stage 14 — Axum API Server](#stage-14--axum-api-server)
- [Stage 15 — WebSocket Event Stream](#stage-15--websocket-event-stream)
- [Stage 16 — React Frontend](#stage-16--react-frontend)

---

## Stage 1 — Project Skeleton

### Step 1.1 — Create the Rust project

```bash
cargo new solanalite
cd solanalite
```

### Step 1.2 — Set up Cargo.toml

Replace the contents of `Cargo.toml`:

```toml
[package]
name = "solanalite"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio       = { version = "1", features = ["full"] }
axum        = { version = "0.7", features = ["ws", "macros"] }
tower-http  = { version = "0.5", features = ["cors"] }
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
uuid        = { version = "1", features = ["v4"] }
sha2        = "0.10"
hex         = "0.4"
thiserror   = "1"
anyhow      = "1"
tracing     = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### Step 1.3 — Verify it compiles

```bash
cargo build
```

Should compile with zero errors.

### Step 1.4 — Set up the file structure

Create all the empty files now so the module declarations don't fail later:

```bash
mkdir -p src/accounts
mkdir -p src/runtime
mkdir -p src/validator
mkdir -p src/api

touch src/config.rs
touch src/types.rs
touch src/error.rs
touch src/poh.rs
touch src/mempool.rs
touch src/network.rs
touch src/scheduler.rs
touch src/consensus.rs
touch src/simulator.rs
touch src/accounts/mod.rs
touch src/accounts/db.rs
touch src/runtime/mod.rs
touch src/runtime/executor.rs
touch src/runtime/builder.rs
touch src/validator/mod.rs
touch src/api/mod.rs
touch src/api/server.rs
touch src/api/ws.rs
```

### Step 1.5 — Declare all modules in main.rs

Replace `src/main.rs`:

```rust
mod config;
mod types;
mod error;
mod poh;
mod mempool;
mod network;
mod scheduler;
mod consensus;
mod simulator;
mod accounts;
mod runtime;
mod validator;
mod api;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("SolanaLite starting...");
    Ok(())
}
```

### Step 1.6 — Add stub pub declarations to each module file

Each file needs at least one `pub` thing or a comment so the module declaration doesn't warn.
Add this to every new `.rs` file you created:

```rust
// TODO
```

And add to each `mod.rs`:

```rust
// pub mod X;  <- will be uncommented as we build
```

### Step 1.7 — Verify it compiles

```bash
cargo build
```

Zero errors. You now have the skeleton. Every future step adds real code into these files.

---

## Stage 2 — Domain Types

> Goal: Every shared data structure in one place. No logic. Just structs.

### Step 2.1 — Write config.rs

```rust
// src/config.rs

#[derive(Debug, Clone)]
pub struct SimConfig {
    pub epoch_length: u64,
    pub slot_duration_ms: u64,
    pub genesis_hash: String,
    pub validators: Vec<ValidatorConfig>,
    pub initial_balances: Vec<(String, u64)>,
}

#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    pub id: u32,
    pub name: String,
    pub stake: u64,
}

impl SimConfig {
    pub fn default() -> Self {
        SimConfig {
            epoch_length: 10,
            slot_duration_ms: 3000,
            genesis_hash: "solanalite-genesis-v1".to_string(),
            validators: vec![
                ValidatorConfig { id: 0, name: "V0".to_string(), stake: 100 },
                ValidatorConfig { id: 1, name: "V1".to_string(), stake: 200 },
                ValidatorConfig { id: 2, name: "V2".to_string(), stake: 100 },
                ValidatorConfig { id: 3, name: "V3".to_string(), stake: 100 },
            ],
            initial_balances: vec![
                ("Alice".to_string(),   100),
                ("Bob".to_string(),      20),
                ("Charlie".to_string(),  50),
            ],
        }
    }
}
```

### Step 2.2 — Write types.rs

```rust
// src/types.rs

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub type ValidatorId = u32;
pub type Pubkey      = String;
pub type SlotNumber  = u64;

// ── Transaction ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id:           String,
    pub from:         Pubkey,
    pub to:           Pubkey,
    pub amount:       u64,
    pub submitted_at: u64,
}

impl Transaction {
    pub fn new(from: &str, to: &str, amount: u64) -> Self {
        Transaction {
            id:           uuid::Uuid::new_v4().to_string(),
            from:         from.to_string(),
            to:           to.to_string(),
            amount,
            submitted_at: unix_ms(),
        }
    }
}

// ── Block ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub slot:          SlotNumber,
    pub epoch:         u64,
    pub leader:        ValidatorId,
    pub transactions:  Vec<Transaction>,
    pub prev_poh_hash: String,
    pub poh_hash:      String,
    pub state_hash:    String,
    pub produced_at:   u64,
}

// ── Vote ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub validator_id: ValidatorId,
    pub block_slot:   SlotNumber,
    pub approved:     bool,
    pub reason:       Option<String>,
}

// ── Network Messages (validator ↔ validator simulation) ───────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    SlotTick { slot: SlotNumber, epoch: u64, leader: ValidatorId },
    NewBlock(Block),
    BlockFinalized { slot: SlotNumber },
}

// ── Sim Events (backend → React frontend via WebSocket) ───────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SimEvent {
    Snapshot {
        balances: HashMap<String, u64>,
        slot:     u64,
    },
    SlotTick {
        slot:         u64,
        epoch:        u64,
        leader:       u32,
        leader_name:  String,
    },
    EpochChange {
        epoch:    u64,
        schedule: Vec<(u64, u32)>,      // (slot_number, leader_id)
    },
    TransactionQueued {
        tx_id:  String,
        from:   String,
        to:     String,
        amount: u64,
    },
    BlockProduced {
        slot:       u64,
        leader:     u32,
        tx_count:   usize,
        poh_hash:   String,
        state_hash: String,
    },
    VoteReceived {
        validator:      u32,
        validator_name: String,
        slot:           u64,
        approved:       bool,
        reason:         Option<String>,
    },
    BlockFinalized {
        slot:      u64,
        approvals: usize,
        total:     usize,
    },
    BalancesUpdated {
        balances: HashMap<String, u64>,
    },
    TransactionResult {
        tx_id:   String,
        success: bool,
        reason:  Option<String>,
    },
}

// ── Helpers ───────────────────────────────────────────────────────────────

pub fn unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
```

### Step 2.3 — Write error.rs

```rust
// src/error.rs

use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum RuntimeError {
    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Insufficient balance in '{account}': have {have}, need {need}")]
    InsufficientBalance { account: String, have: u64, need: u64 },

    #[error("Cannot transfer zero amount")]
    ZeroAmount,

    #[error("Cannot transfer to yourself")]
    SelfTransfer,

    #[error("Account locked: {0}")]
    AccountLocked(String),

    #[error("Compute budget exceeded")]
    ComputeBudgetExceeded,
}

#[derive(Debug, Error)]
pub enum MempoolError {
    #[error("Mempool is full (capacity: {0})")]
    Full(usize),
}
```

### Step 2.4 — Verify

```bash
cargo build
```

---

## Stage 3 — AccountsDB

> Goal: A key-value store for account balances with snapshot/rollback support.

### Step 3.1 — Write accounts/db.rs

```rust
// src/accounts/db.rs

use std::collections::HashMap;
use sha2::{Sha256, Digest};
use crate::error::RuntimeError;

#[derive(Debug, Clone)]
pub struct AccountsDb {
    accounts: HashMap<String, u64>,
    snapshot: Option<HashMap<String, u64>>,
}

impl AccountsDb {
    pub fn new(initial: &[(String, u64)]) -> Self {
        let accounts = initial.iter().cloned().collect();
        AccountsDb { accounts, snapshot: None }
    }

    // ── Reads ──────────────────────────────────────────────────────────────

    pub fn balance(&self, key: &str) -> Option<u64> {
        self.accounts.get(key).copied()
    }

    pub fn all_balances(&self) -> HashMap<String, u64> {
        self.accounts.clone()
    }

    pub fn exists(&self, key: &str) -> bool {
        self.accounts.contains_key(key)
    }

    // ── Writes ─────────────────────────────────────────────────────────────

    pub fn ensure_account(&mut self, key: &str) {
        self.accounts.entry(key.to_string()).or_insert(0);
    }

    pub fn debit(&mut self, key: &str, amount: u64) -> Result<(), RuntimeError> {
        let bal = self.accounts.get_mut(key)
            .ok_or_else(|| RuntimeError::AccountNotFound(key.to_string()))?;

        if *bal < amount {
            return Err(RuntimeError::InsufficientBalance {
                account: key.to_string(),
                have: *bal,
                need: amount,
            });
        }

        *bal -= amount;
        Ok(())
    }

    pub fn credit(&mut self, key: &str, amount: u64) {
        *self.accounts.entry(key.to_string()).or_insert(0) += amount;
    }

    // ── Snapshot / Rollback ────────────────────────────────────────────────

    pub fn snapshot(&mut self) {
        self.snapshot = Some(self.accounts.clone());
    }

    pub fn commit(&mut self) {
        self.snapshot = None;
    }

    pub fn rollback(&mut self) {
        if let Some(snap) = self.snapshot.take() {
            self.accounts = snap;
        }
    }

    // ── State fingerprint ──────────────────────────────────────────────────

    pub fn state_hash(&self) -> String {
        // Sort keys — HashMap iteration is random, must sort for determinism
        let mut pairs: Vec<(&String, &u64)> = self.accounts.iter().collect();
        pairs.sort_by_key(|(k, _)| k.as_str());

        let mut hasher = Sha256::new();
        for (key, value) in pairs {
            hasher.update(key.as_bytes());
            hasher.update(b":");
            hasher.update(value.to_le_bytes());
            hasher.update(b";");
        }

        hex::encode(hasher.finalize())
    }
}
```

### Step 3.2 — Expose from accounts/mod.rs

```rust
// src/accounts/mod.rs
pub mod db;
pub use db::AccountsDb;
```

### Step 3.3 — Write unit tests (at bottom of db.rs)

```rust
// append to src/accounts/db.rs

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> AccountsDb {
        AccountsDb::new(&[
            ("Alice".to_string(), 100),
            ("Bob".to_string(), 20),
        ])
    }

    #[test]
    fn test_balance() {
        let db = test_db();
        assert_eq!(db.balance("Alice"), Some(100));
        assert_eq!(db.balance("Nobody"), None);
    }

    #[test]
    fn test_debit_credit() {
        let mut db = test_db();
        db.debit("Alice", 30).unwrap();
        db.credit("Bob", 30);
        assert_eq!(db.balance("Alice"), Some(70));
        assert_eq!(db.balance("Bob"), Some(50));
    }

    #[test]
    fn test_insufficient_balance() {
        let mut db = test_db();
        let result = db.debit("Alice", 200);
        assert!(result.is_err());
        assert_eq!(db.balance("Alice"), Some(100)); // unchanged
    }

    #[test]
    fn test_snapshot_and_rollback() {
        let mut db = test_db();
        db.snapshot();
        db.debit("Alice", 50).unwrap();
        assert_eq!(db.balance("Alice"), Some(50));
        db.rollback();
        assert_eq!(db.balance("Alice"), Some(100)); // restored
    }

    #[test]
    fn test_snapshot_and_commit() {
        let mut db = test_db();
        db.snapshot();
        db.debit("Alice", 50).unwrap();
        db.commit();
        assert_eq!(db.balance("Alice"), Some(50)); // change kept
    }

    #[test]
    fn test_state_hash_is_deterministic() {
        let db1 = AccountsDb::new(&[("Alice".to_string(), 90), ("Bob".to_string(), 30)]);
        let db2 = AccountsDb::new(&[("Bob".to_string(), 30), ("Alice".to_string(), 90)]);
        // Different insertion order — must produce same hash
        assert_eq!(db1.state_hash(), db2.state_hash());
    }

    #[test]
    fn test_state_hash_changes_on_mutation() {
        let db1 = test_db();
        let mut db2 = test_db();
        db2.debit("Alice", 1).unwrap();
        assert_ne!(db1.state_hash(), db2.state_hash());
    }
}
```

### Step 3.4 — Run tests

```bash
cargo test accounts
```

All 7 tests must pass before continuing.

---

## Stage 4 — PoH Chain

> Goal: A SHA-256 hash chain that links every slot and every transaction in order.

### Step 4.1 — Write poh.rs

```rust
// src/poh.rs

use sha2::{Sha256, Digest};
use crate::types::Block;

#[derive(Debug, Clone)]
pub struct PohChain {
    pub current_hash: String,
    pub slot:         u64,
}

impl PohChain {
    pub fn new(genesis: &str) -> Self {
        let hash = hex::encode(Sha256::digest(genesis.as_bytes()));
        PohChain { current_hash: hash, slot: 0 }
    }

    /// Advance the chain by one slot.
    /// Returns the hash BEFORE transactions — stored as block.prev_poh_hash.
    pub fn advance_slot(&mut self, slot: u64) -> String {
        let input = format!("{}:slot:{}", self.current_hash, slot);
        self.current_hash = hex::encode(Sha256::digest(input.as_bytes()));
        self.slot = slot;
        self.current_hash.clone()
    }

    /// Fold one successful transaction into the chain.
    /// Called once per included tx during block production.
    pub fn record_transaction(&mut self, tx_id: &str) {
        let input = format!("{}:tx:{}", self.current_hash, tx_id);
        self.current_hash = hex::encode(Sha256::digest(input.as_bytes()));
    }

    /// Called by non-leader validators to verify a received block.
    /// Replays the hash operations and checks the final hash matches.
    pub fn verify_block(block: &Block) -> bool {
        // Start from prev_poh_hash and replay slot advance
        let after_slot = hex::encode(Sha256::digest(
            format!("{}:slot:{}", block.prev_poh_hash, block.slot).as_bytes()
        ));

        // Replay each transaction fold
        let mut hash = after_slot;
        for tx in &block.transactions {
            hash = hex::encode(Sha256::digest(
                format!("{}:tx:{}", hash, tx.id).as_bytes()
            ));
        }

        // Final hash must match block's recorded poh_hash
        hash == block.poh_hash
    }
}
```

### Step 4.2 — Write tests (append to poh.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Transaction, unix_ms};

    #[test]
    fn test_chain_advances() {
        let mut poh = PohChain::new("genesis");
        let h0 = poh.current_hash.clone();
        poh.advance_slot(1);
        assert_ne!(poh.current_hash, h0);
    }

    #[test]
    fn test_deterministic() {
        let mut poh1 = PohChain::new("genesis");
        let mut poh2 = PohChain::new("genesis");
        poh1.advance_slot(1);
        poh2.advance_slot(1);
        assert_eq!(poh1.current_hash, poh2.current_hash);
    }

    #[test]
    fn test_verify_empty_block() {
        let mut poh = PohChain::new("genesis");
        let prev = poh.advance_slot(1);

        let block = Block {
            slot: 1,
            epoch: 0,
            leader: 0,
            transactions: vec![],
            prev_poh_hash: prev,
            poh_hash: poh.current_hash.clone(),
            state_hash: "".to_string(),
            produced_at: unix_ms(),
        };

        assert!(PohChain::verify_block(&block));
    }

    #[test]
    fn test_verify_block_with_tx() {
        let mut poh = PohChain::new("genesis");
        let prev = poh.advance_slot(1);

        let tx = Transaction::new("Alice", "Bob", 10);
        poh.record_transaction(&tx.id);

        let block = Block {
            slot: 1,
            epoch: 0,
            leader: 0,
            transactions: vec![tx],
            prev_poh_hash: prev,
            poh_hash: poh.current_hash.clone(),
            state_hash: "".to_string(),
            produced_at: unix_ms(),
        };

        assert!(PohChain::verify_block(&block));
    }

    #[test]
    fn test_tampered_block_fails_verify() {
        let mut poh = PohChain::new("genesis");
        let prev = poh.advance_slot(1);
        let tx = Transaction::new("Alice", "Bob", 10);
        poh.record_transaction(&tx.id);

        let mut block = Block {
            slot: 1,
            epoch: 0,
            leader: 0,
            transactions: vec![tx],
            prev_poh_hash: prev,
            poh_hash: poh.current_hash.clone(),
            state_hash: "".to_string(),
            produced_at: unix_ms(),
        };

        // Tamper with the poh hash
        block.poh_hash = "000000000000000000000000000000000000000000000000000000000000dead".to_string();
        assert!(!PohChain::verify_block(&block));
    }
}
```

### Step 4.3 — Run tests

```bash
cargo test poh
```

All 5 tests pass.

---

## Stage 5 — Runtime Executor

> Goal: Execute a single transaction against AccountsDB with locking, snapshotting, and rollback.

### Step 5.1 — Write AccountLocks in executor.rs

```rust
// src/runtime/executor.rs

use std::collections::{HashMap, HashSet};
use crate::accounts::AccountsDb;
use crate::error::RuntimeError;
use crate::types::Transaction;

// ── Account Locks ─────────────────────────────────────────────────────────

#[derive(Debug, Default)]
struct AccountLocks {
    write_locks: HashSet<String>,
}

impl AccountLocks {
    fn try_acquire_write(&mut self, key: &str) -> bool {
        if self.write_locks.contains(key) {
            return false; // already locked
        }
        self.write_locks.insert(key.to_string());
        true
    }

    fn release_write(&mut self, key: &str) {
        self.write_locks.remove(key);
    }
}
```

### Step 5.2 — Write ExecutionResult

```rust
// append to src/runtime/executor.rs

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub tx_id:             String,
    pub success:           bool,
    pub compute_units_used: u64,
    pub error:             Option<RuntimeError>,
}

impl ExecutionResult {
    fn ok(tx_id: &str, compute: u64) -> Self {
        ExecutionResult {
            tx_id: tx_id.to_string(),
            success: true,
            compute_units_used: compute,
            error: None,
        }
    }

    fn fail(tx_id: &str, e: RuntimeError) -> Self {
        ExecutionResult {
            tx_id: tx_id.to_string(),
            success: false,
            compute_units_used: 0,
            error: Some(e),
        }
    }
}
```

### Step 5.3 — Write RuntimeExecutor

```rust
// append to src/runtime/executor.rs

const COMPUTE_UNIT_LIMIT: u64 = 200_000;

#[derive(Debug, Default)]
pub struct RuntimeExecutor {
    locks: AccountLocks,
}

impl RuntimeExecutor {
    pub fn new() -> Self {
        RuntimeExecutor { locks: AccountLocks::default() }
    }

    pub fn execute(&mut self, tx: &Transaction, db: &mut AccountsDb) -> ExecutionResult {
        // Step 1: Basic validation
        if tx.amount == 0 {
            return ExecutionResult::fail(&tx.id, RuntimeError::ZeroAmount);
        }
        if tx.from == tx.to {
            return ExecutionResult::fail(&tx.id, RuntimeError::SelfTransfer);
        }

        // Step 2: Ensure destination account exists
        db.ensure_account(&tx.to);

        // Step 3: Acquire write locks on both accounts
        if !self.locks.try_acquire_write(&tx.from) {
            return ExecutionResult::fail(&tx.id, RuntimeError::AccountLocked(tx.from.clone()));
        }
        if !self.locks.try_acquire_write(&tx.to) {
            self.locks.release_write(&tx.from); // release what we already took
            return ExecutionResult::fail(&tx.id, RuntimeError::AccountLocked(tx.to.clone()));
        }

        // Step 4: Snapshot current state
        db.snapshot();

        // Step 5: Execute transfer and meter compute
        let result = Self::do_transfer(tx, db);

        // Step 6: Commit or rollback based on result
        match result {
            Ok(compute) => {
                db.commit();
                self.locks.release_write(&tx.from);
                self.locks.release_write(&tx.to);
                ExecutionResult::ok(&tx.id, compute)
            }
            Err(e) => {
                db.rollback(); // state restored to snapshot
                self.locks.release_write(&tx.from);
                self.locks.release_write(&tx.to);
                ExecutionResult::fail(&tx.id, e)
            }
        }
    }

    fn do_transfer(tx: &Transaction, db: &mut AccountsDb) -> Result<u64, RuntimeError> {
        let mut compute = 0u64;

        // Load sender balance
        let from_bal = db.balance(&tx.from)
            .ok_or_else(|| RuntimeError::AccountNotFound(tx.from.clone()))?;
        compute += 100;

        // Check balance
        if from_bal < tx.amount {
            return Err(RuntimeError::InsufficientBalance {
                account: tx.from.clone(),
                have: from_bal,
                need: tx.amount,
            });
        }
        compute += 50;

        // Debit sender
        db.debit(&tx.from, tx.amount)?;
        compute += 200;

        // Credit receiver
        db.credit(&tx.to, tx.amount);
        compute += 200;

        // Check compute budget
        if compute > COMPUTE_UNIT_LIMIT {
            return Err(RuntimeError::ComputeBudgetExceeded);
        }

        Ok(compute)
    }
}
```

### Step 5.4 — Expose from runtime/mod.rs

```rust
// src/runtime/mod.rs
pub mod executor;
pub mod builder;
pub use executor::{RuntimeExecutor, ExecutionResult};
```

### Step 5.5 — Write tests (append to executor.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::AccountsDb;
    use crate::types::Transaction;

    fn db() -> AccountsDb {
        AccountsDb::new(&[("Alice".to_string(), 100), ("Bob".to_string(), 20)])
    }

    #[test]
    fn test_successful_transfer() {
        let mut exec = RuntimeExecutor::new();
        let mut db = db();
        let tx = Transaction::new("Alice", "Bob", 10);
        let result = exec.execute(&tx, &mut db);
        assert!(result.success);
        assert_eq!(db.balance("Alice"), Some(90));
        assert_eq!(db.balance("Bob"), Some(30));
    }

    #[test]
    fn test_insufficient_balance_rolls_back() {
        let mut exec = RuntimeExecutor::new();
        let mut db = db();
        let tx = Transaction::new("Alice", "Bob", 9999);
        let result = exec.execute(&tx, &mut db);
        assert!(!result.success);
        assert_eq!(db.balance("Alice"), Some(100)); // unchanged
        assert_eq!(db.balance("Bob"), Some(20));    // unchanged
    }

    #[test]
    fn test_zero_amount_rejected() {
        let mut exec = RuntimeExecutor::new();
        let mut db = db();
        let tx = Transaction::new("Alice", "Bob", 0);
        let result = exec.execute(&tx, &mut db);
        assert!(!result.success);
    }

    #[test]
    fn test_self_transfer_rejected() {
        let mut exec = RuntimeExecutor::new();
        let mut db = db();
        let tx = Transaction::new("Alice", "Alice", 10);
        let result = exec.execute(&tx, &mut db);
        assert!(!result.success);
        assert_eq!(db.balance("Alice"), Some(100)); // unchanged
    }

    #[test]
    fn test_creates_new_account_for_receiver() {
        let mut exec = RuntimeExecutor::new();
        let mut db = db();
        let tx = Transaction::new("Alice", "Dave", 15); // Dave doesn't exist
        let result = exec.execute(&tx, &mut db);
        assert!(result.success);
        assert_eq!(db.balance("Dave"), Some(15)); // created and credited
    }

    #[test]
    fn test_account_not_found() {
        let mut exec = RuntimeExecutor::new();
        let mut db = db();
        let tx = Transaction::new("Nobody", "Bob", 10);
        let result = exec.execute(&tx, &mut db);
        assert!(!result.success);
    }
}
```

### Step 5.6 — Run tests

```bash
cargo test runtime::executor
```

All 6 pass.

---

## Stage 6 — Block Builder

> Goal: Takes a batch of transactions, runs them through the executor, folds into PoH, returns a Block.

### Step 6.1 — Write runtime/builder.rs

```rust
// src/runtime/builder.rs

use crate::accounts::AccountsDb;
use crate::poh::PohChain;
use crate::runtime::executor::{RuntimeExecutor, ExecutionResult};
use crate::types::{Block, Transaction, ValidatorId, unix_ms};

pub struct BlockBuilder {
    executor: RuntimeExecutor,
}

impl BlockBuilder {
    pub fn new() -> Self {
        BlockBuilder { executor: RuntimeExecutor::new() }
    }

    pub fn build(
        &mut self,
        slot: u64,
        epoch: u64,
        leader: ValidatorId,
        transactions: Vec<Transaction>,
        poh: &mut PohChain,
        db: &mut AccountsDb,
    ) -> (Block, Vec<ExecutionResult>) {

        // Advance PoH for this slot — captures prev_poh_hash
        let prev_poh_hash = poh.advance_slot(slot);

        let mut included_txs = vec![];
        let mut results       = vec![];

        for tx in transactions {
            let result = self.executor.execute(&tx, db);

            if result.success {
                // Only successful transactions are folded into PoH
                poh.record_transaction(&tx.id);
                included_txs.push(tx);
            }

            results.push(result);
        }

        let block = Block {
            slot,
            epoch,
            leader,
            prev_poh_hash,
            poh_hash:   poh.current_hash.clone(),
            state_hash: db.state_hash(),
            transactions: included_txs,
            produced_at: unix_ms(),
        };

        (block, results)
    }
}
```

### Step 6.2 — Write tests (append to builder.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::AccountsDb;
    use crate::poh::PohChain;
    use crate::types::Transaction;

    #[test]
    fn test_build_empty_block() {
        let mut builder = BlockBuilder::new();
        let mut db  = AccountsDb::new(&[("Alice".to_string(), 100)]);
        let mut poh = PohChain::new("genesis");

        let (block, results) = builder.build(1, 0, 0, vec![], &mut poh, &mut db);

        assert_eq!(block.slot, 1);
        assert_eq!(block.transactions.len(), 0);
        assert!(results.is_empty());
        // PoH chain must be verifiable
        assert!(PohChain::verify_block(&block));
    }

    #[test]
    fn test_build_block_with_successful_tx() {
        let mut builder = BlockBuilder::new();
        let mut db  = AccountsDb::new(&[
            ("Alice".to_string(), 100),
            ("Bob".to_string(), 20),
        ]);
        let mut poh = PohChain::new("genesis");
        let tx = Transaction::new("Alice", "Bob", 10);

        let (block, results) = builder.build(1, 0, 0, vec![tx], &mut poh, &mut db);

        assert_eq!(block.transactions.len(), 1);
        assert!(results[0].success);
        assert_eq!(db.balance("Alice"), Some(90));
        assert_eq!(db.balance("Bob"), Some(30));
        assert!(PohChain::verify_block(&block));
    }

    #[test]
    fn test_failed_tx_not_included_in_block() {
        let mut builder = BlockBuilder::new();
        let mut db  = AccountsDb::new(&[("Alice".to_string(), 5)]);
        let mut poh = PohChain::new("genesis");
        let tx = Transaction::new("Alice", "Bob", 100); // too much

        let (block, results) = builder.build(1, 0, 0, vec![tx], &mut poh, &mut db);

        assert_eq!(block.transactions.len(), 0); // not included
        assert!(!results[0].success);
        assert_eq!(db.balance("Alice"), Some(5)); // unchanged
        assert!(PohChain::verify_block(&block));
    }

    #[test]
    fn test_state_hash_reflects_execution() {
        let mut builder = BlockBuilder::new();
        let mut db1 = AccountsDb::new(&[("Alice".to_string(), 100), ("Bob".to_string(), 20)]);
        let mut db2 = AccountsDb::new(&[("Alice".to_string(), 100), ("Bob".to_string(), 20)]);
        let mut poh1 = PohChain::new("genesis");
        let mut poh2 = PohChain::new("genesis");

        let tx = Transaction::new("Alice", "Bob", 10);
        let (block1, _) = builder.build(1, 0, 0, vec![tx.clone()], &mut poh1, &mut db1);

        // Different builder, same tx — must produce same state hash
        let mut builder2 = BlockBuilder::new();
        let (block2, _) = builder2.build(1, 0, 0, vec![tx], &mut poh2, &mut db2);

        assert_eq!(block1.state_hash, block2.state_hash);
        assert_eq!(block1.poh_hash, block2.poh_hash);
    }
}
```

### Step 6.3 — Run tests

```bash
cargo test runtime::builder
```

All 4 pass.

---

## Stage 7 — Mempool

> Goal: A thread-safe FIFO queue. API pushes in. Leader drains out.

### Step 7.1 — Write mempool.rs

```rust
// src/mempool.rs

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::error::MempoolError;
use crate::types::Transaction;

const DEFAULT_CAPACITY: usize = 1000;

#[derive(Clone)]
pub struct Mempool {
    inner:    Arc<Mutex<VecDeque<Transaction>>>,
    capacity: usize,
}

impl Mempool {
    pub fn new(capacity: usize) -> Self {
        Mempool {
            inner: Arc::new(Mutex::new(VecDeque::new())),
            capacity,
        }
    }

    pub async fn push(&self, tx: Transaction) -> Result<(), MempoolError> {
        let mut q = self.inner.lock().await;
        if q.len() >= self.capacity {
            return Err(MempoolError::Full(self.capacity));
        }
        q.push_back(tx);
        Ok(())
    }

    /// Pull up to `max` transactions — called by the leader each slot.
    pub async fn drain(&self, max: usize) -> Vec<Transaction> {
        let mut q = self.inner.lock().await;
        let count = max.min(q.len());
        q.drain(..count).collect()
    }

    pub async fn len(&self) -> usize {
        self.inner.lock().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.inner.lock().await.is_empty()
    }
}
```

### Step 7.2 — Write tests (append to mempool.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Transaction;

    #[tokio::test]
    async fn test_push_and_drain() {
        let mp = Mempool::new(100);
        mp.push(Transaction::new("A", "B", 1)).await.unwrap();
        mp.push(Transaction::new("A", "B", 2)).await.unwrap();
        mp.push(Transaction::new("A", "B", 3)).await.unwrap();

        let batch = mp.drain(2).await;
        assert_eq!(batch.len(), 2);
        assert_eq!(mp.len().await, 1); // one left
    }

    #[tokio::test]
    async fn test_capacity_limit() {
        let mp = Mempool::new(2);
        mp.push(Transaction::new("A", "B", 1)).await.unwrap();
        mp.push(Transaction::new("A", "B", 2)).await.unwrap();
        let result = mp.push(Transaction::new("A", "B", 3)).await;
        assert!(result.is_err()); // full
    }

    #[tokio::test]
    async fn test_drain_more_than_available() {
        let mp = Mempool::new(100);
        mp.push(Transaction::new("A", "B", 1)).await.unwrap();
        let batch = mp.drain(50).await; // ask for 50, only 1 available
        assert_eq!(batch.len(), 1);
    }
}
```

### Step 7.3 — Run tests

```bash
cargo test mempool
```

All 3 pass.

---

## Stage 8 — Leader Scheduler

> Goal: Epoch-based, stake-weighted, deterministically-shuffled slot → leader mapping.

### Step 8.1 — Write scheduler.rs

```rust
// src/scheduler.rs

use crate::config::{SimConfig, ValidatorConfig};
use crate::types::ValidatorId;

pub struct LeaderScheduler {
    epoch_length:      u64,
    current_epoch:     u64,
    schedule:          Vec<ValidatorId>,
    validator_configs: Vec<ValidatorConfig>,
}

impl LeaderScheduler {
    pub fn new(config: &SimConfig) -> Self {
        let mut s = LeaderScheduler {
            epoch_length:      config.epoch_length,
            current_epoch:     0,
            schedule:          vec![],
            validator_configs: config.validators.clone(),
        };
        s.build_schedule(0);
        s
    }

    pub fn leader_for_slot(&self, slot: u64) -> ValidatorId {
        let idx = (slot % self.epoch_length) as usize;
        self.schedule[idx % self.schedule.len()]
    }

    pub fn is_epoch_boundary(&self, slot: u64) -> bool {
        slot > 0 && slot % self.epoch_length == 0
    }

    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    /// Called at epoch boundary. Returns (slot → leader) mapping for the new epoch.
    pub fn advance_epoch(&mut self, new_epoch: u64) -> Vec<(u64, ValidatorId)> {
        self.build_schedule(new_epoch);
        self.current_epoch = new_epoch;

        let epoch_start = new_epoch * self.epoch_length;
        self.schedule.iter().enumerate()
            .map(|(i, &v)| (epoch_start + i as u64, v))
            .collect()
    }

    fn build_schedule(&mut self, epoch: u64) {
        let total_stake: u64 = self.validator_configs.iter().map(|v| v.stake).sum();

        let mut slots: Vec<ValidatorId> = Vec::new();
        for v in &self.validator_configs {
            let count = ((v.stake as f64 / total_stake as f64)
                * self.epoch_length as f64).round() as usize;
            for _ in 0..count {
                slots.push(v.id);
            }
        }

        // Pad to exact epoch_length if rounding left us short
        while slots.len() < self.epoch_length as usize {
            slots.push(self.validator_configs[0].id);
        }
        slots.truncate(self.epoch_length as usize);

        // Deterministic shuffle — same epoch seed = same result on every machine
        Self::lcg_shuffle(&mut slots, epoch);

        self.schedule = slots;
    }

    fn lcg_shuffle(slots: &mut Vec<ValidatorId>, seed: u64) {
        let mut s = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        for i in (1..slots.len()).rev() {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let j = (s >> 33) as usize % (i + 1);
            slots.swap(i, j);
        }
    }
}
```

### Step 8.2 — Write tests (append to scheduler.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{SimConfig, ValidatorConfig};

    fn make_config(stakes: &[u64]) -> SimConfig {
        SimConfig {
            epoch_length: 10,
            slot_duration_ms: 100,
            genesis_hash: "test".to_string(),
            validators: stakes.iter().enumerate().map(|(i, &s)| ValidatorConfig {
                id: i as u32,
                name: format!("V{}", i),
                stake: s,
            }).collect(),
            initial_balances: vec![],
        }
    }

    #[test]
    fn test_equal_stakes_round_robin_distribution() {
        let config = make_config(&[100, 100, 100, 100]);
        let sched = LeaderScheduler::new(&config);
        // Each validator should appear exactly 10/4 = 2-3 times in epoch of 10
        for id in 0..4u32 {
            let count = (0..10).filter(|&s| sched.leader_for_slot(s) == id).count();
            assert!(count >= 2 && count <= 3, "V{} had {} slots", id, count);
        }
    }

    #[test]
    fn test_stake_weighted_distribution() {
        // V1 has 2x stake — should get 2x slots
        let config = make_config(&[100, 200, 100, 100]);
        let sched = LeaderScheduler::new(&config);
        let v1_count = (0..10).filter(|&s| sched.leader_for_slot(s) == 1).count();
        assert_eq!(v1_count, 4); // 200/500 * 10 = 4 slots
    }

    #[test]
    fn test_same_epoch_same_schedule() {
        let config = make_config(&[100, 200, 100, 100]);
        let s1 = LeaderScheduler::new(&config);
        let s2 = LeaderScheduler::new(&config);
        for slot in 0..10 {
            assert_eq!(s1.leader_for_slot(slot), s2.leader_for_slot(slot));
        }
    }

    #[test]
    fn test_epoch_boundary_detection() {
        let config = make_config(&[100, 100, 100, 100]);
        let sched = LeaderScheduler::new(&config);
        assert!(!sched.is_epoch_boundary(0));
        assert!(!sched.is_epoch_boundary(5));
        assert!(sched.is_epoch_boundary(10));
        assert!(sched.is_epoch_boundary(20));
    }

    #[test]
    fn test_different_epochs_different_schedules() {
        let config = make_config(&[100, 200, 100, 100]);
        let mut sched = LeaderScheduler::new(&config);
        let epoch0: Vec<u32> = (0..10).map(|s| sched.leader_for_slot(s)).collect();
        sched.advance_epoch(1);
        let epoch1: Vec<u32> = (0..10).map(|s| sched.leader_for_slot(s)).collect();
        // Different order (shuffled with different seed), but same leader counts
        assert_ne!(epoch0, epoch1);
        let v1_e0 = epoch0.iter().filter(|&&v| v == 1).count();
        let v1_e1 = epoch1.iter().filter(|&&v| v == 1).count();
        assert_eq!(v1_e0, v1_e1); // count stays the same, order changes
    }
}
```

### Step 8.3 — Run tests

```bash
cargo test scheduler
```

All 5 pass.

---

## Stage 9 — Network Bus

> Goal: Simulated broadcast channel that acts as the P2P network layer.

### Step 9.1 — Write network.rs

```rust
// src/network.rs

use tokio::sync::broadcast;
use crate::types::NetworkMessage;

const BUS_CAPACITY: usize = 128;

#[derive(Clone)]
pub struct NetworkBus {
    sender: broadcast::Sender<NetworkMessage>,
}

impl NetworkBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(BUS_CAPACITY);
        NetworkBus { sender }
    }

    /// Get a new receiver — call once per validator before spawning its task
    pub fn subscribe(&self) -> broadcast::Receiver<NetworkMessage> {
        self.sender.subscribe()
    }

    /// Broadcast a message to all current subscribers
    pub fn send(&self, msg: NetworkMessage) {
        self.sender.send(msg).ok(); // ok() — ignore if no subscribers
    }

    pub fn sender(&self) -> broadcast::Sender<NetworkMessage> {
        self.sender.clone()
    }
}
```

No tests needed here — it's a thin wrapper. The integration tests in Stage 12 will validate it.

---

## Stage 10 — Slot Clock

> Goal: A Tokio task that ticks every N milliseconds, advances slots, detects epoch boundaries, and broadcasts SlotTick to all validators.

### Step 10.1 — Write the slot clock task in simulator.rs

```rust
// src/simulator.rs
// We'll fill this file progressively. Start with just the clock.

use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, Duration, MissedTickBehavior};

use crate::config::SimConfig;
use crate::network::NetworkBus;
use crate::scheduler::LeaderScheduler;
use crate::types::{NetworkMessage, SimEvent};

pub async fn run_slot_clock(
    config:    Arc<SimConfig>,
    network:   NetworkBus,
    scheduler: Arc<RwLock<LeaderScheduler>>,
    event_tx:  broadcast::Sender<SimEvent>,
) {
    let mut ticker = interval(Duration::from_millis(config.slot_duration_ms));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let mut slot = 0u64;

    loop {
        ticker.tick().await;
        slot += 1;

        // Check epoch boundary
        let (epoch, schedule_announcement) = {
            let mut sched = scheduler.write().await;

            if sched.is_epoch_boundary(slot) {
                let new_epoch = slot / config.epoch_length;
                let new_schedule = sched.advance_epoch(new_epoch);
                (new_epoch, Some(new_schedule))
            } else {
                (sched.current_epoch(), None)
            }
        };

        // Announce epoch change to frontend
        if let Some(schedule) = schedule_announcement {
            event_tx.send(SimEvent::EpochChange { epoch, schedule }).ok();
            println!("[Clock] Epoch {} begins at slot {}", epoch, slot);
        }

        // Get the leader for this slot
        let leader = {
            let sched = scheduler.read().await;
            sched.leader_for_slot(slot)
        };

        println!("[Clock] Slot {} | Epoch {} | Leader: V{}", slot, epoch, leader);

        // Broadcast to all validators
        network.send(NetworkMessage::SlotTick { slot, epoch, leader });

        // Emit to frontend
        event_tx.send(SimEvent::SlotTick {
            slot,
            epoch,
            leader,
            leader_name: format!("V{}", leader),
        }).ok();
    }
}
```

### Step 10.2 — Quick test: run the clock alone in main.rs

Temporarily replace `main()` to see it ticking:

```rust
// src/main.rs — TEMPORARY TEST, will be replaced

use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config    = Arc::new(config::SimConfig::default());
    let network   = network::NetworkBus::new();
    let scheduler = Arc::new(RwLock::new(scheduler::LeaderScheduler::new(&config)));
    let (event_tx, _) = broadcast::channel(256);

    simulator::run_slot_clock(config, network, scheduler, event_tx).await;

    Ok(())
}
```

```bash
cargo run
```

You should see:
```
[Clock] Slot 1 | Epoch 0 | Leader: V1
[Clock] Slot 2 | Epoch 0 | Leader: V3
...
[Clock] Epoch 1 begins at slot 10
[Clock] Slot 10 | Epoch 1 | Leader: V0
```

Every 3 seconds. Ctrl+C to stop. **This is working.** Restore main.rs back to the minimal stub.

---

## Stage 11 — Single Validator Loop

> Goal: One validator that listens to the network bus and produces or validates blocks.

### Step 11.1 — Write validator/mod.rs

Write the full Validator struct and its event loop. This is the biggest single step.

```rust
// src/validator/mod.rs

use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use crate::accounts::AccountsDb;
use crate::config::ValidatorConfig;
use crate::mempool::Mempool;
use crate::network::NetworkBus;
use crate::poh::PohChain;
use crate::runtime::builder::BlockBuilder;
use crate::runtime::executor::RuntimeExecutor;
use crate::scheduler::LeaderScheduler;
use crate::types::{Block, NetworkMessage, SimEvent, Transaction, Vote};

pub struct Validator {
    pub id:             u32,
    pub name:           String,
    pub stake:          u64,
    pub finalized_slot: u64,

    // Core systems (owned by this validator)
    poh:     PohChain,
    builder: BlockBuilder,

    // Shared state (Arc handles)
    accounts:  Arc<RwLock<AccountsDb>>,
    mempool:   Mempool,
    scheduler: Arc<RwLock<LeaderScheduler>>,

    // Outbound communication
    network:  NetworkBus,
    vote_tx:  mpsc::Sender<Vote>,
    event_tx: broadcast::Sender<SimEvent>,
    block_tx: mpsc::Sender<Block>,  // sends produced block to consensus
}

impl Validator {
    pub fn new(
        vc:        &ValidatorConfig,
        genesis:   &str,
        accounts:  Arc<RwLock<AccountsDb>>,
        mempool:   Mempool,
        scheduler: Arc<RwLock<LeaderScheduler>>,
        network:   NetworkBus,
        vote_tx:   mpsc::Sender<Vote>,
        block_tx:  mpsc::Sender<Block>,
        event_tx:  broadcast::Sender<SimEvent>,
    ) -> Self {
        Validator {
            id: vc.id, name: vc.name.clone(), stake: vc.stake,
            finalized_slot: 0,
            poh:     PohChain::new(genesis),
            builder: BlockBuilder::new(),
            accounts, mempool, scheduler, network,
            vote_tx, block_tx, event_tx,
        }
    }

    // ── Main event loop ───────────────────────────────────────────────────

    pub async fn run(mut self, mut net_rx: broadcast::Receiver<NetworkMessage>) {
        loop {
            match net_rx.recv().await {
                Ok(NetworkMessage::SlotTick { slot, epoch, leader }) => {
                    if leader == self.id {
                        self.on_leader_slot(slot, epoch).await;
                    }
                    // Non-leaders: wait for NewBlock
                }
                Ok(NetworkMessage::NewBlock(block)) => {
                    if block.leader != self.id {
                        self.on_received_block(block).await;
                    }
                }
                Ok(NetworkMessage::BlockFinalized { slot }) => {
                    self.finalized_slot = slot;
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("[{}] lagged by {} messages — catching up", self.name, n);
                }
                Err(_) => {
                    println!("[{}] network channel closed, shutting down", self.name);
                    break;
                }
            }
        }
    }

    // ── Leader path ───────────────────────────────────────────────────────

    async fn on_leader_slot(&mut self, slot: u64, epoch: u64) {
        println!("[{}] I am leader for slot {}", self.name, slot);

        // Pull transactions from mempool
        let txs = self.mempool.drain(10).await;
        println!("[{}] Drained {} txs from mempool", self.name, txs.len());

        // Build block — acquires write lock on AccountsDb during execution
        let (block, results) = {
            let mut db = self.accounts.write().await;
            self.builder.build(slot, epoch, self.id, txs, &mut self.poh, &mut db)
        }; // write lock released here

        // Report each tx result to frontend
        for r in &results {
            self.emit(SimEvent::TransactionResult {
                tx_id:   r.tx_id.clone(),
                success: r.success,
                reason:  r.error.as_ref().map(|e| e.to_string()),
            });
        }

        // Report block production to frontend
        self.emit(SimEvent::BlockProduced {
            slot,
            leader:     self.id,
            tx_count:   block.transactions.len(),
            poh_hash:   block.poh_hash.clone(),
            state_hash: block.state_hash.clone(),
        });

        println!("[{}] Produced block: slot={} txs={} poh={}...",
            self.name, slot, block.transactions.len(), &block.poh_hash[..8]);

        // Send block to consensus engine for vote tracking
        self.block_tx.send(block.clone()).await.ok();

        // Broadcast to other validators
        self.network.send(NetworkMessage::NewBlock(block));
    }

    // ── Validator path ─────────────────────────────────────────────────────

    async fn on_received_block(&mut self, block: Block) {
        println!("[{}] Received block for slot {} from V{}",
            self.name, block.slot, block.leader);

        let approved = self.validate_block(&block).await;

        let reason = if !approved {
            Some("PoH verification or state hash mismatch".to_string())
        } else {
            None
        };

        println!("[{}] Vote on slot {}: {}", self.name, block.slot,
            if approved { "APPROVED ✓" } else { "REJECTED ✗" });

        self.emit(SimEvent::VoteReceived {
            validator:      self.id,
            validator_name: self.name.clone(),
            slot:           block.slot,
            approved,
            reason:         reason.clone(),
        });

        let vote = Vote {
            validator_id: self.id,
            block_slot:   block.slot,
            approved,
            reason,
        };

        self.vote_tx.send(vote).await.ok();
    }

    async fn validate_block(&self, block: &Block) -> bool {
        // Check 1: PoH chain integrity
        if !PohChain::verify_block(block) {
            eprintln!("[{}] PoH verification FAILED for slot {}", self.name, block.slot);
            return false;
        }

        // Check 2: Re-execute transactions and compare state hash
        // Read current shared state, clone it for local re-execution
        let mut local_db = {
            let db = self.accounts.read().await;
            db.clone()
        };

        let mut executor = RuntimeExecutor::new();
        for tx in &block.transactions {
            executor.execute(tx, &mut local_db);
        }

        let expected_hash = local_db.state_hash();
        if expected_hash != block.state_hash {
            eprintln!("[{}] State hash MISMATCH for slot {} — expected {}, got {}",
                self.name, block.slot, &expected_hash[..8], &block.state_hash[..8]);
            return false;
        }

        true
    }

    // ── Helper ─────────────────────────────────────────────────────────────

    fn emit(&self, event: SimEvent) {
        self.event_tx.send(event).ok();
    }
}
```

### Step 11.2 — Test single validator in main.rs (temporary)

```rust
// src/main.rs — TEMPORARY, watch one validator work

use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg    = Arc::new(config::SimConfig::default());
    let bus    = network::NetworkBus::new();
    let sched  = Arc::new(RwLock::new(scheduler::LeaderScheduler::new(&cfg)));
    let db     = Arc::new(RwLock::new(accounts::AccountsDb::new(&cfg.initial_balances)));
    let pool   = mempool::Mempool::new(100);
    let (event_tx, _) = broadcast::channel::<types::SimEvent>(256);
    let (vote_tx, _vote_rx)   = mpsc::channel::<types::Vote>(64);
    let (block_tx, _block_rx) = mpsc::channel::<types::Block>(32);

    // Pre-fill mempool with one transaction
    pool.push(types::Transaction::new("Alice", "Bob", 10)).await?;

    // Spawn ONE validator (V0 — may or may not be the leader each slot)
    let vc  = cfg.validators[0].clone();
    let v0  = validator::Validator::new(
        &vc, &cfg.genesis_hash,
        Arc::clone(&db), pool.clone(),
        Arc::clone(&sched), bus.clone(),
        vote_tx, block_tx, event_tx.clone(),
    );
    let rx0 = bus.subscribe();
    tokio::spawn(async move { v0.run(rx0).await });

    // Run the slot clock
    simulator::run_slot_clock(cfg, bus, sched, event_tx).await;

    Ok(())
}
```

```bash
cargo run
```

You should see slot ticks. When V0 is the leader you'll see block production and tx execution. When it's not the leader, nothing happens (no one else exists yet to send a block). That's fine — the test is "does a validator produce a block when it's the leader?" **Yes. Move on.**

---

## Stage 12 — Four Validators + Full Simulation Loop

> Goal: All 4 validators running. Blocks produced, received, validated, and voted on.

### Step 12.1 — Write the Simulator orchestrator

Add to `simulator.rs` below the slot clock function:

```rust
// append to src/simulator.rs

use crate::accounts::AccountsDb;
use crate::mempool::Mempool;
use crate::types::{Block, Vote};
use crate::validator::Validator;

#[derive(Clone)]
pub struct AppState {
    pub mempool:   Mempool,
    pub event_tx:  broadcast::Sender<SimEvent>,
    pub accounts:  Arc<RwLock<AccountsDb>>,
    pub chain:     Arc<RwLock<Vec<Block>>>,
    pub config:    Arc<SimConfig>,
}

pub async fn run(config: SimConfig) -> anyhow::Result<AppState> {
    let config = Arc::new(config);

    // ── Shared state ──────────────────────────────────────────────────────
    let accounts = Arc::new(RwLock::new(AccountsDb::new(&config.initial_balances)));
    let chain:   Arc<RwLock<Vec<Block>>> = Arc::new(RwLock::new(vec![]));
    let mempool  = Mempool::new(1000);

    // ── Channels ──────────────────────────────────────────────────────────
    let bus                       = NetworkBus::new();
    let (vote_tx,  vote_rx)       = mpsc::channel::<Vote>(64);
    let (block_tx, block_rx)      = mpsc::channel::<Block>(32);
    let (event_tx, _)             = broadcast::channel::<SimEvent>(256);

    // ── Scheduler ─────────────────────────────────────────────────────────
    let scheduler = Arc::new(RwLock::new(LeaderScheduler::new(&config)));

    // ── Spawn Slot Clock ──────────────────────────────────────────────────
    {
        let cfg  = Arc::clone(&config);
        let net  = bus.clone();
        let sch  = Arc::clone(&scheduler);
        let evt  = event_tx.clone();
        tokio::spawn(async move { run_slot_clock(cfg, net, sch, evt).await });
    }

    // ── Spawn Validators ──────────────────────────────────────────────────
    for vc in &config.validators {
        let v = Validator::new(
            vc,
            &config.genesis_hash,
            Arc::clone(&accounts),
            mempool.clone(),
            Arc::clone(&scheduler),
            bus.clone(),
            vote_tx.clone(),
            block_tx.clone(),
            event_tx.clone(),
        );
        let rx = bus.subscribe();
        tokio::spawn(async move { v.run(rx).await });
    }

    // ── Spawn Consensus Engine ────────────────────────────────────────────
    {
        let engine = consensus::ConsensusEngine::new(
            config.validators.len(),
            Arc::clone(&accounts),
            Arc::clone(&chain),
            bus.clone(),
            event_tx.clone(),
        );
        tokio::spawn(async move { engine.run(vote_rx, block_rx).await });
    }

    Ok(AppState { mempool, event_tx, accounts, chain, config })
}
```

### Step 12.2 — Update main.rs to run the full simulation

```rust
// src/main.rs

mod config; mod types; mod error; mod poh; mod mempool;
mod network; mod scheduler; mod consensus; mod simulator;
mod accounts; mod runtime; mod validator; mod api;

use config::SimConfig;
use types::Transaction;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg   = SimConfig::default();
    let state = simulator::run(cfg).await?;

    // Submit a couple of test transactions
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    state.mempool.push(Transaction::new("Alice", "Bob", 10)).await?;
    state.mempool.push(Transaction::new("Charlie", "Alice", 5)).await?;

    // Run forever
    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");
    Ok(())
}
```

### Step 12.3 — Run and observe

```bash
cargo run
```

Expected terminal output pattern:
```
[Clock]  Slot 1 | Epoch 0 | Leader: V1
[V1]     I am leader for slot 1
[V1]     Drained 2 txs from mempool
[V1]     Produced block: slot=1 txs=2 poh=a4f8b2c1...
[V0]     Received block for slot 1 from V1
[V0]     Vote on slot 1: APPROVED ✓
[V2]     Received block for slot 1 from V1
[V2]     Vote on slot 1: APPROVED ✓
[V3]     Received block for slot 1 from V1
[V3]     Vote on slot 1: APPROVED ✓

[Clock]  Slot 2 | Epoch 0 | Leader: V3
[V3]     I am leader for slot 2
[V3]     Drained 0 txs from mempool
[V3]     Produced block: slot=2 txs=0 poh=9c211f7a...
...
```

**The core simulation is working.** Every system up to this point is exercised. This is the milestone.

---

## Stage 13 — Consensus Engine

> Goal: Collect votes, detect 2/3 threshold, commit finalized state to AccountsDB.

### Step 13.1 — Write consensus.rs

```rust
// src/consensus.rs

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

use crate::accounts::AccountsDb;
use crate::network::NetworkBus;
use crate::runtime::executor::RuntimeExecutor;
use crate::types::{Block, NetworkMessage, SimEvent, SlotNumber, ValidatorId, Vote};

pub struct ConsensusEngine {
    total:         usize,
    votes:         HashMap<SlotNumber, SlotVotes>,
    pending:       HashMap<SlotNumber, Block>,
    accounts:      Arc<RwLock<AccountsDb>>,
    chain:         Arc<RwLock<Vec<Block>>>,
    network:       NetworkBus,
    event_tx:      broadcast::Sender<SimEvent>,
}

#[derive(Default)]
struct SlotVotes {
    approvals:   HashSet<ValidatorId>,
    rejections:  HashSet<ValidatorId>,
}

impl ConsensusEngine {
    pub fn new(
        total:    usize,
        accounts: Arc<RwLock<AccountsDb>>,
        chain:    Arc<RwLock<Vec<Block>>>,
        network:  NetworkBus,
        event_tx: broadcast::Sender<SimEvent>,
    ) -> Self {
        ConsensusEngine {
            total, votes: HashMap::new(), pending: HashMap::new(),
            accounts, chain, network, event_tx,
        }
    }

    pub async fn run(
        mut self,
        mut vote_rx:  mpsc::Receiver<Vote>,
        mut block_rx: mpsc::Receiver<Block>,
    ) {
        loop {
            tokio::select! {
                Some(block) = block_rx.recv() => {
                    self.pending.insert(block.slot, block);
                }
                Some(vote) = vote_rx.recv() => {
                    self.handle_vote(vote).await;
                }
            }
        }
    }

    async fn handle_vote(&mut self, vote: Vote) {
        let slot_votes = self.votes.entry(vote.block_slot).or_default();

        if vote.approved {
            slot_votes.approvals.insert(vote.validator_id);
        } else {
            slot_votes.rejections.insert(vote.validator_id);
        }

        let approvals = slot_votes.approvals.len();
        let threshold = (self.total * 2) / 3 + 1;

        println!("[Consensus] Slot {} — {}/{} approvals (need {})",
            vote.block_slot, approvals, self.total, threshold);

        if approvals >= threshold {
            self.finalize(vote.block_slot).await;
        }
    }

    async fn finalize(&mut self, slot: SlotNumber) {
        let block = match self.pending.remove(&slot) {
            Some(b) => b,
            None    => return,
        };

        println!("[Consensus] Block {} FINALIZED", slot);

        // Commit block's transactions to shared AccountsDB
        let final_balances = {
            let mut db = self.accounts.write().await;
            let mut exec = RuntimeExecutor::new();
            for tx in &block.transactions {
                exec.execute(tx, &mut db);
            }
            db.all_balances()
        };

        // Append to chain history
        self.chain.write().await.push(block.clone());

        let approvals = self.votes.get(&slot)
            .map(|v| v.approvals.len()).unwrap_or(0);

        // Broadcast finalization to all validators
        self.network.send(NetworkMessage::BlockFinalized { slot });

        // Push events to frontend
        self.event_tx.send(SimEvent::BlockFinalized {
            slot,
            approvals,
            total: self.total,
        }).ok();

        self.event_tx.send(SimEvent::BalancesUpdated {
            balances: final_balances,
        }).ok();

        // Clean up
        self.votes.remove(&slot);
    }
}
```

### Step 13.2 — Run again and verify finalization appears

```bash
cargo run
```

Now you should see after the votes:
```
[Consensus] Slot 1 — 1/4 approvals (need 3)
[Consensus] Slot 1 — 2/4 approvals (need 3)
[Consensus] Slot 1 — 3/4 approvals (need 3)
[Consensus] Block 1 FINALIZED
```

The full cycle works end to end. Ctrl+C to stop.

---

## Stage 14 — Axum API Server

> Goal: REST endpoints so the React app can submit transactions and read state.

### Step 14.1 — Write api/server.rs

```rust
// src/api/server.rs

use axum::{Router, routing::{get, post}, extract::State, Json, http::StatusCode};
use tower_http::cors::CorsLayer;
use serde::{Serialize, Deserialize};

use crate::simulator::AppState;
use crate::types::{Transaction, Block};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health",     get(health))
        .route("/tx",         post(submit_tx))
        .route("/balances",   get(get_balances))
        .route("/chain",      get(get_chain))
        .route("/validators", get(get_validators))
        .route("/ws",         get(crate::api::ws::ws_handler))
        .with_state(state)
        .layer(CorsLayer::permissive())
}

// GET /health
async fn health() -> &'static str { "ok" }

// POST /tx
#[derive(Deserialize)]
pub struct TransferRequest {
    pub from:   String,
    pub to:     String,
    pub amount: u64,
}

#[derive(Serialize)]
pub struct TxResponse {
    pub tx_id: String,
    pub queued: bool,
    pub message: String,
}

async fn submit_tx(
    State(state): State<AppState>,
    Json(req): Json<TransferRequest>,
) -> (StatusCode, Json<TxResponse>) {
    let tx    = Transaction::new(&req.from, &req.to, req.amount);
    let tx_id = tx.id.clone();

    // Notify frontend of queued tx
    state.event_tx.send(crate::types::SimEvent::TransactionQueued {
        tx_id: tx_id.clone(),
        from:  req.from.clone(),
        to:    req.to.clone(),
        amount: req.amount,
    }).ok();

    match state.mempool.push(tx).await {
        Ok(()) => (
            StatusCode::ACCEPTED,
            Json(TxResponse { tx_id, queued: true, message: "Transaction queued".into() }),
        ),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(TxResponse { tx_id, queued: false, message: e.to_string() }),
        ),
    }
}

// GET /balances
async fn get_balances(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db = state.accounts.read().await;
    Json(serde_json::json!({ "balances": db.all_balances() }))
}

// GET /chain
async fn get_chain(State(state): State<AppState>) -> Json<Vec<Block>> {
    let chain = state.chain.read().await;
    Json(chain.clone())
}

// GET /validators
async fn get_validators(State(state): State<AppState>) -> Json<serde_json::Value> {
    let validators: Vec<_> = state.config.validators.iter().map(|v| {
        serde_json::json!({ "id": v.id, "name": v.name, "stake": v.stake })
    }).collect();
    Json(serde_json::json!({ "validators": validators }))
}
```

### Step 14.2 — Write api/mod.rs and api/ws.rs stub

```rust
// src/api/mod.rs
pub mod server;
pub mod ws;
```

```rust
// src/api/ws.rs — stub for now, filled in Stage 15
use axum::response::IntoResponse;
use axum::extract::{State, WebSocketUpgrade};
use crate::simulator::AppState;

pub async fn ws_handler(
    _ws: WebSocketUpgrade,
    _state: State<AppState>,
) -> impl IntoResponse {
    "WebSocket not yet implemented"
}
```

### Step 14.3 — Start the server in main.rs

```rust
// src/main.rs — update to start both simulation and API

use config::SimConfig;
use types::Transaction;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg   = SimConfig::default();
    let state = simulator::run(cfg).await?;

    // Seed mempool with test transactions
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    state.mempool.push(Transaction::new("Alice", "Bob", 10)).await?;

    // Start Axum server
    let app      = api::server::build_router(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    println!("API server running on http://localhost:3001");
    axum::serve(listener, app).await?;

    Ok(())
}
```

### Step 14.4 — Test the endpoints

```bash
cargo run

# New terminal:
curl http://localhost:3001/health
# → ok

curl http://localhost:3001/balances
# → {"balances":{"Alice":100,"Bob":20,"Charlie":50}}

curl -X POST http://localhost:3001/tx \
  -H "Content-Type: application/json" \
  -d '{"from":"Alice","to":"Bob","amount":15}'
# → {"tx_id":"...","queued":true,"message":"Transaction queued"}

# Wait a few seconds, then:
curl http://localhost:3001/balances
# → {"balances":{"Alice":85,"Bob":35,"Charlie":50}}

curl http://localhost:3001/chain
# → [{ "slot":1, "leader":1, ... }]
```

All 5 endpoints respond correctly.

---

## Stage 15 — WebSocket Event Stream

> Goal: React frontend receives all simulation events in real time.

### Step 15.1 — Write the real ws_handler in api/ws.rs

```rust
// src/api/ws.rs

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use tokio::select;

use crate::simulator::AppState;
use crate::types::SimEvent;

pub async fn ws_handler(
    ws:    WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // 1. Send current state snapshot immediately on connect
    let snapshot = {
        let db    = state.accounts.read().await;
        let chain = state.chain.read().await;
        SimEvent::Snapshot {
            balances: db.all_balances(),
            slot:     chain.len() as u64,
        }
    };
    let snap_json = serde_json::to_string(&snapshot).unwrap();
    if socket.send(Message::Text(snap_json)).await.is_err() {
        return; // client disconnected immediately
    }

    // 2. Subscribe to live events and stream them
    let mut events = state.event_tx.subscribe();

    loop {
        select! {
            result = events.recv() => {
                match result {
                    Ok(event) => {
                        let json = serde_json::to_string(&event).unwrap();
                        if socket.send(Message::Text(json)).await.is_err() {
                            break; // client disconnected
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        eprintln!("[WS] Client lagged by {} events", n);
                    }
                    Err(_) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}
```

### Step 15.2 — Test with wscat

```bash
# Install wscat if needed: npm install -g wscat
cargo run &
wscat -c ws://localhost:3001/ws
```

You should see a JSON snapshot immediately, then a stream of events:
```json
{"type":"Snapshot","data":{"balances":{"Alice":100,"Bob":20,"Charlie":50},"slot":0}}
{"type":"SlotTick","data":{"slot":1,"epoch":0,"leader":1,"leader_name":"V1"}}
{"type":"BlockProduced","data":{"slot":1,"leader":1,"tx_count":0,...}}
{"type":"VoteReceived","data":{"validator":0,"slot":1,"approved":true,...}}
{"type":"VoteReceived","data":{"validator":2,"slot":1,"approved":true,...}}
{"type":"VoteReceived","data":{"validator":3,"slot":1,"approved":true,...}}
{"type":"BlockFinalized","data":{"slot":1,"approvals":3,"total":4}}
{"type":"BalancesUpdated","data":{"balances":{"Alice":100,"Bob":20,"Charlie":50}}}
```

The Rust backend is completely done. Everything after this is the React frontend.

---

## Stage 16 — React Frontend

> Build in this exact order. Each component is independently testable.

### Step 16.1 — Create Next.js app

```bash
cd .. # go to project root (alongside solanalite/)
npx create-next-app@latest frontend --typescript --tailwind --app --no-src-dir
cd frontend
```

### Step 16.2 — Write the WebSocket hook

`frontend/hooks/useSimulation.ts`:

```typescript
import { useState, useEffect, useRef, useCallback } from 'react';

// ── Types ──────────────────────────────────────────────────────────────────

export type SimEvent =
  | { type: 'Snapshot';         data: { balances: Record<string,number>; slot: number } }
  | { type: 'SlotTick';         data: { slot: number; epoch: number; leader: number; leader_name: string } }
  | { type: 'EpochChange';      data: { epoch: number; schedule: [number,number][] } }
  | { type: 'TransactionQueued';data: { tx_id: string; from: string; to: string; amount: number } }
  | { type: 'BlockProduced';    data: { slot: number; leader: number; tx_count: number; poh_hash: string; state_hash: string } }
  | { type: 'VoteReceived';     data: { validator: number; validator_name: string; slot: number; approved: boolean; reason?: string } }
  | { type: 'BlockFinalized';   data: { slot: number; approvals: number; total: number } }
  | { type: 'BalancesUpdated';  data: { balances: Record<string,number> } }
  | { type: 'TransactionResult';data: { tx_id: string; success: boolean; reason?: string } };

interface ValidatorDisplay { id: number; name: string; stake: number; isLeader: boolean; lastVote: 'approved'|'rejected'|null; lastVoteSlot: number }
interface BlockDisplay { slot: number; leader: number; tx_count: number; state_hash: string; poh_hash: string; finalized: boolean }

export interface SimState {
  connected:     boolean;
  slot:          number;
  epoch:         number;
  leader:        number;
  balances:      Record<string, number>;
  prevBalances:  Record<string, number>;
  validators:    ValidatorDisplay[];
  blocks:        BlockDisplay[];
  events:        SimEvent[];
  pendingTxIds:  Set<string>;
  confirmedTxIds:Set<string>;
}

// ── Hook ───────────────────────────────────────────────────────────────────

export function useSimulation(validatorConfigs: {id:number;name:string;stake:number}[]) {
  const ws = useRef<WebSocket|null>(null);

  const [state, setState] = useState<SimState>({
    connected: false, slot: 0, epoch: 0, leader: 0,
    balances: {}, prevBalances: {},
    validators: validatorConfigs.map(v => ({
      ...v, isLeader: false, lastVote: null, lastVoteSlot: 0
    })),
    blocks: [], events: [], pendingTxIds: new Set(), confirmedTxIds: new Set(),
  });

  useEffect(() => {
    const connect = () => {
      ws.current = new WebSocket('ws://localhost:3001/ws');

      ws.current.onopen  = () => setState(s => ({ ...s, connected: true }));
      ws.current.onclose = () => {
        setState(s => ({ ...s, connected: false }));
        setTimeout(connect, 2000); // auto-reconnect
      };

      ws.current.onmessage = (e: MessageEvent) => {
        const event: SimEvent = JSON.parse(e.data);

        setState(prev => {
          const next = { ...prev };
          next.events = [event, ...prev.events].slice(0, 150);

          switch (event.type) {

            case 'Snapshot':
              next.balances     = event.data.balances;
              next.prevBalances = event.data.balances;
              next.slot         = event.data.slot;
              break;

            case 'SlotTick':
              next.slot   = event.data.slot;
              next.epoch  = event.data.epoch;
              next.leader = event.data.leader;
              next.validators = prev.validators.map(v => ({
                ...v, isLeader: v.id === event.data.leader
              }));
              break;

            case 'VoteReceived':
              next.validators = prev.validators.map(v =>
                v.id === event.data.validator
                  ? { ...v, lastVote: event.data.approved ? 'approved' : 'rejected', lastVoteSlot: event.data.slot }
                  : v
              );
              break;

            case 'BlockProduced':
              next.blocks = [
                { slot: event.data.slot, leader: event.data.leader,
                  tx_count: event.data.tx_count, state_hash: event.data.state_hash,
                  poh_hash: event.data.poh_hash, finalized: false },
                ...prev.blocks
              ].slice(0, 50);
              break;

            case 'BlockFinalized':
              next.blocks = prev.blocks.map(b =>
                b.slot === event.data.slot ? { ...b, finalized: true } : b
              );
              break;

            case 'BalancesUpdated':
              next.prevBalances = prev.balances;
              next.balances     = event.data.balances;
              break;

            case 'TransactionQueued':
              next.pendingTxIds = new Set([...prev.pendingTxIds, event.data.tx_id]);
              break;

            case 'TransactionResult':
              if (event.data.success) {
                const pending = new Set(prev.pendingTxIds);
                pending.delete(event.data.tx_id);
                const confirmed = new Set([...prev.confirmedTxIds, event.data.tx_id]);
                next.pendingTxIds   = pending;
                next.confirmedTxIds = confirmed;
              }
              break;
          }
          return next;
        });
      };
    };

    connect();
    return () => ws.current?.close();
  }, []);

  const submitTransfer = useCallback(async (from: string, to: string, amount: number) => {
    const res = await fetch('http://localhost:3001/tx', {
      method:  'POST',
      headers: { 'Content-Type': 'application/json' },
      body:    JSON.stringify({ from, to, amount }),
    });
    return res.json() as Promise<{ tx_id: string; queued: boolean; message: string }>;
  }, []);

  return { ...state, submitTransfer };
}
```

### Step 16.3 — Build each component (in order)

Build one, verify it renders, move to next.

**SlotHeader** — `frontend/components/SlotHeader.tsx`
Shows: Slot | Epoch | Leader | Connected status

**ValidatorGrid** — `frontend/components/ValidatorGrid.tsx`
4 cards. Active leader glows. Shows stake, last vote, last vote slot.

**BalanceTable** — `frontend/components/BalanceTable.tsx`
Accounts with balance. Colour-flash on change (red = decrease, green = increase).

**BlockFeed** — `frontend/components/BlockFeed.tsx`
Scrollable list of blocks. FINALIZED badge in green. Shows tx count, leader, PoH hash (truncated).

**EventLog** — `frontend/components/EventLog.tsx`
Monospace dark terminal. Maps each SimEvent type to an emoji + description line. Auto-scrolls.

**TransactionForm** — `frontend/components/TransactionForm.tsx`
Dropdown for From/To (Alice/Bob/Charlie). Number input for amount. Submit button calls `submitTransfer`.

### Step 16.4 — Wire everything in app/page.tsx

```typescript
// frontend/app/page.tsx
'use client';
import { useSimulation } from '../hooks/useSimulation';
import SlotHeader       from '../components/SlotHeader';
import ValidatorGrid    from '../components/ValidatorGrid';
import BalanceTable     from '../components/BalanceTable';
import BlockFeed        from '../components/BlockFeed';
import EventLog         from '../components/EventLog';
import TransactionForm  from '../components/TransactionForm';

const VALIDATOR_CONFIGS = [
  { id: 0, name: 'V0', stake: 100 },
  { id: 1, name: 'V1', stake: 200 },
  { id: 2, name: 'V2', stake: 100 },
  { id: 3, name: 'V3', stake: 100 },
];

export default function Dashboard() {
  const sim = useSimulation(VALIDATOR_CONFIGS);

  return (
    <main className="min-h-screen bg-gray-950 text-white p-6">
      <SlotHeader
        slot={sim.slot} epoch={sim.epoch}
        leaderName={sim.validators.find(v => v.isLeader)?.name ?? '—'}
        connected={sim.connected}
      />
      <div className="grid grid-cols-4 gap-4 mt-6">
        {sim.validators.map(v => <ValidatorCard key={v.id} validator={v} />)}
      </div>
      <div className="grid grid-cols-3 gap-6 mt-6">
        <BalanceTable balances={sim.balances} prevBalances={sim.prevBalances} />
        <BlockFeed blocks={sim.blocks} />
        <TransactionForm onSubmit={sim.submitTransfer} />
      </div>
      <EventLog events={sim.events} />
    </main>
  );
}
```

### Step 16.5 — Run everything

```bash
# Terminal 1 — Rust backend
cd solanalite && cargo run

# Terminal 2 — React frontend
cd frontend && npm run dev

# Browser
open http://localhost:3000
```

---

## Final Checklist

Before calling it done, verify every item manually:

```
□ cargo test  — all tests pass
□ Slot clock ticks every 3 seconds in terminal
□ Epoch boundary fires at slot 10, 20, 30... with new schedule
□ Leader rotation matches stake weights (V1 leads ~40% of slots)
□ Block produced every slot with correct PoH hash
□ All 3 non-leader validators vote after each block
□ Consensus fires exactly when 3rd approval arrives
□ Balance state committed correctly after finalization
□ POST /tx → appears in next leader's block
□ GET /balances → updates after each finalization
□ GET /chain → growing list of blocks
□ WebSocket → snapshot on connect, then live events
□ React: slot counter ticks in browser
□ React: leader card highlights each slot
□ React: balances update live after finalization
□ React: block feed grows with each finalized block
□ React: event log streams all activity
□ React: submit form → tx confirmed within ~6 seconds
□ V1 visibly leads more slots than V0/V2/V3 over 30 slots
```