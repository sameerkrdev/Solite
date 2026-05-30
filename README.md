# Solite — Complete Project Documentation

> **For developers, judges, teachers, and AI models.**
> This document is the single source of truth for what Solite is, how it is built, how every part works, and how every part connects to every other part.

---

## Table of Contents

1. [What is Solite](#1-what-is-solite)
2. [Two Systems in One Binary](#2-two-systems-in-one-binary)
3. [Full Architecture Diagram](#3-full-architecture-diagram)
4. [Technology Stack](#4-technology-stack)
5. [Custom Phantom — Identity System](#5-custom-phantom--identity-system)
6. [Mini Solana Runtime — Execution Engine](#6-mini-solana-runtime--execution-engine)
7. [How the Two Systems Communicate](#7-how-the-two-systems-communicate)
8. [API Layer — All Routes](#8-api-layer--all-routes)
9. [Complete Folder Structure](#9-complete-folder-structure)
10. [Every Data Structure](#10-every-data-structure)
11. [Step by Step — User Journeys](#11-step-by-step--user-journeys)
12. [Step by Step — Transaction Lifecycle](#12-step-by-step--transaction-lifecycle)
13. [Data Security Model](#13-data-security-model)
14. [Error Handling](#14-error-handling)
15. [Frontend Architecture](#15-frontend-architecture)
16. [Build Order](#16-build-order)
17. [Running the Project](#17-running-the-project)
18. [Key Design Decisions Explained](#18-key-design-decisions-explained)
19. [How to Answer Judge Questions](#19-how-to-answer-judge-questions)

---

## 1. What is Solite

Solite is a **single Rust binary** that does two things simultaneously:

**Thing 1 — Custom Phantom Wallet**
A user authentication and wallet management system that mirrors how the real Phantom wallet works. Users register, generate Ed25519 keypairs, get BIP39 recovery phrases, encrypt their private keys with a password, and sign transactions — all without any private key ever touching the server.

**Thing 2 — Mini Solana Runtime**
A simulation of the internal architecture of a Solana validator cluster. Four validators run as concurrent async tasks, produce blocks on a slot clock, validate each other's blocks, vote on them, reach consensus, and finalize state — all with a live dashboard streamed to the frontend via WebSocket.

**What Solite demonstrates:**

- How real blockchain wallets manage keys without storing private keys on a server
- Ed25519 keypair generation, BIP39 recovery phrases, AES-256-GCM encryption
- Proof of History (PoH) — a cryptographic hash chain that orders events
- Epoch-based, stake-weighted leader scheduling
- Parallel transaction execution with account locking and snapshot/rollback
- 2/3 supermajority consensus finalization
- Live real-time observability via WebSocket

**What Solite is NOT:**

- Not a real P2P network — Tokio channels simulate network message passing
- Not real Solana PoH (VDF) — a SHA-256 hash chain driven by a slot clock
- Not a production node — a simulator that mirrors the architecture accurately

---

## 2. Two Systems in One Binary

```
┌─────────────────────────────────────────────────────────────────────┐
│                         SINGLE RUST BINARY                          │
│                                                                     │
│  ┌──────────────────────────┐      ┌───────────────────────────┐   │
│  │     CUSTOM PHANTOM       │      │    MINI SOLANA RUNTIME    │   │
│  │                          │      │                           │   │
│  │  Who you are             │      │  What happens to balances │   │
│  │  What keys you own       │      │  when you act             │   │
│  │  Proving you can act     │      │                           │   │
│  │                          │      │  4 Validators             │   │
│  │  User accounts           │      │  PoH chain                │   │
│  │  Keypair generation      │      │  Leader scheduler         │   │
│  │  Recovery phrases        │      │  Slot clock               │   │
│  │  Password / Google auth  │      │  Mempool                  │   │
│  │  JWT tokens              │      │  Consensus engine         │   │
│  │  Encrypt private keys    │      │  AccountsDb               │   │
│  │                          │      │  Vec<Block>               │   │
│  │  Owns: UserDb            │      │                           │   │
│  │                          │      │  Owns: AccountsDb         │   │
│  │                          │      │        Vec<Block>         │   │
│  └──────────┬───────────────┘      └───────────────┬───────────┘   │
│             │                                      │               │
│             │         THREE TOKIO CHANNELS         │               │
│             │                                      │               │
│             ├── runtime_query_tx ────────────────► │               │
│             │   + oneshot response ◄────────────── │               │
│             │                                      │               │
│             ├── mempool_tx ──────────────────────► │               │
│             │                                      │               │
│             │ ◄── event_tx (broadcast) ─────────── │               │
│             │                                      │               │
└─────────────┼──────────────────────────────────────┼───────────────┘
              │                                      │
              └──────────────┬───────────────────────┘
                             │
                    SINGLE AXUM SERVER
                    port 3001
                    two handler groups
                             │
              ┌──────────────┴──────────────┐
              │                             │
        /auth/*  /user/*             simulation/*
        (JWT protected)              (open read / JWT write)
```

**The address (base58 pubkey) is the only link between both systems.**
Phantom stores it as a string reference in `WalletEntry.address`.
Runtime uses it as the key in `AccountsDb` HashMap.
That string is the bridge.

---

## 3. Full Architecture Diagram

```
BROWSER (React + Bun + Vite)
│
│  Onboarding.tsx      — register, login, create wallet
│  WalletPanel.tsx     — address, balance, tx history
│  SlotHeader.tsx      — slot, epoch, leader
│  ValidatorCard.tsx   — stake, state, last vote (x4)
│  BalanceTable.tsx    — live balances with delta animation
│  BlockFeed.tsx       — finalized block list
│  EventLog.tsx        — terminal event stream
│  TransactionForm.tsx — sign + submit transaction
│
│  store/wallet.ts       (zustand) — user session + wallets
│  store/simulation.ts   (zustand) — slot, balances, blocks
│  hooks/useWebSocket.ts — connects to /ws, feeds stores
│  lib/wallet.ts         — @noble/ed25519 keypair + signing
│  lib/api.ts            — typed fetch wrappers
│
├── HTTP POST /auth/register, /auth/login, /tx, /airdrop
├── HTTP GET  /balances, /chain, /validators, /wallet/:address
└── WebSocket ws://localhost:3001/ws
       │
       ▼
──────────────────────────────────────────────────────────────
AXUM SERVER — port 3001
──────────────────────────────────────────────────────────────
       │
       ├── api/router.rs         — assembles all routes + CORS
       ├── api/state.rs          — AppState (channels only)
       ├── api/middleware/auth.rs — JWT extractor
       ├── api/ws.rs             — WebSocket upgrade + stream
       │
       ├── api/handlers/phantom/
       │   ├── register.rs       POST /auth/register
       │   ├── login.rs          POST /auth/login
       │   ├── refresh.rs        POST /auth/refresh
       │   ├── google_register.rs POST /auth/google/register
       │   ├── google_login.rs   POST /auth/google/login
       │   ├── wallets.rs        GET  /user/wallets
       │   ├── wallet_new.rs     POST /user/wallet/new
       │   ├── wallet_import.rs  POST /user/wallet/import
       │   ├── recovery.rs       GET  /user/recovery-phrase
       │   ├── privkey.rs        GET  /user/private-key
       │   └── sign.rs           POST /user/sign
       │
       └── api/handlers/simulation/
           ├── tx.rs             POST /tx
           ├── airdrop.rs        POST /airdrop
           ├── balances.rs       GET  /balances
           ├── chain.rs          GET  /chain
           ├── wallet_info.rs    GET  /wallet/:address
           └── validators.rs     GET  /validators
       │
       │   AppState contains:
       │     runtime_query_tx  — mpsc::Sender<RuntimeQuery>
       │     mempool_tx        — mpsc::Sender<Transaction>
       │     event_tx          — broadcast::Sender<SimEvent>
       │     users             — Arc<RwLock<UserDb>>
       │
──────────────────────────────────────────────────────────────
                    THREE TOKIO CHANNELS
──────────────────────────────────────────────────────────────
       │
       │  runtime_query_tx (mpsc) + oneshot response
       │  mempool_tx (mpsc)
       │  event_tx (broadcast)
       │
──────────────────────────────────────────────────────────────
PHANTOM SYSTEM                   RUNTIME SYSTEM
src/phantom/                     src/runtime/
──────────────────────────────────────────────────────────────
                                 │
                                 ├── simulator.rs
                                 │     select! loop:
                                 │       slot ticks
                                 │       query_rx (runtime queries)
                                 │
                                 ├── slot clock task
                                 │     tokio::interval(3000ms)
                                 │     → NetworkMessage::SlotTick broadcast
                                 │
                                 ├── validator tasks (x4)
                                 │     select! on network_rx:
                                 │       SlotTick:
                                 │         if leader → produce_block()
                                 │         else     → wait for NewBlock
                                 │       NewBlock:
                                 │         verify PoH + state hash
                                 │         send Vote via vote_tx
                                 │       BlockFinalized:
                                 │         update finalized_slot
                                 │
                                 ├── consensus task
                                 │     select! on vote_rx + block_rx:
                                 │       collect votes per slot
                                 │       when approvals >= (2/3 + 1):
                                 │         commit txs to AccountsDb
                                 │         append to Vec<Block>
                                 │         broadcast BlockFinalized
                                 │         emit BalancesUpdated
                                 │
                                 ├── accounts/db.rs
                                 │     HashMap<address, balance>
                                 │     snapshot / rollback / commit
                                 │     state_hash()
                                 │
                                 ├── executor/executor.rs
                                 │     verify → lock → snapshot
                                 │     → execute → commit/rollback
                                 │     → unlock
                                 │
                                 ├── executor/builder.rs
                                 │     advance PoH slot
                                 │     execute txs
                                 │     fold successful txs into PoH
                                 │     assemble Block
                                 │
                                 ├── poh.rs
                                 │     sha256 hash chain
                                 │     advance_slot() / record_transaction()
                                 │     verify_block()
                                 │
                                 ├── mempool.rs
                                 │     Arc<Mutex<VecDeque<Transaction>>>
                                 │     push() / drain()
                                 │
                                 ├── scheduler.rs
                                 │     stake-weighted slot assignment
                                 │     LCG deterministic shuffle per epoch
                                 │
                                 └── network.rs
                                       broadcast::channel wrapper
                                       simulate P2P gossip
```

---

## 4. Technology Stack

### Backend

| Crate | Version | Purpose |
|---|---|---|
| `tokio` | 1 full | Async runtime, channels, timers |
| `axum` | 0.7 ws+macros | HTTP server + WebSocket upgrade |
| `tower-http` | 0.5 cors+trace | CORS, request tracing |
| `serde` + `serde_json` | 1 | JSON serialisation everywhere |
| `ed25519-dalek` | 2 rand_core | Ed25519 keypair, signing, verification |
| `bs58` | 0.5 | Base58 encode/decode (wallet addresses) |
| `sha2` | 0.10 | SHA-256 for PoH chain + state hash |
| `hex` | 0.4 | Hex encode/decode for hashes + signatures |
| `uuid` | 1 v4 | Random transaction IDs |
| `argon2` | 0.5 | Password hashing (Argon2id) + KDF for AES key |
| `aes-gcm` | 0.10 | AES-256-GCM encrypt/decrypt private keys |
| `bip39` | 2 | BIP39 recovery phrase generation/validation |
| `jsonwebtoken` | 9 | JWT access + refresh token issue/verify |
| `oauth2` | 4 | Google OAuth2 (optional) |
| `thiserror` | 1 | Typed error enums |
| `anyhow` | 1 | Top-level error propagation |
| `tracing` + `tracing-subscriber` | 0.1 / 0.3 | Structured logging |
| `chrono` | 0.4 serde | Timestamps |
| `rand` | 0.8 | Randomness for keypair generation |

### Frontend

| Package | Purpose |
|---|---|
| Bun | Runtime + package manager + bundler |
| Vite | Dev server with proxy to :3001 |
| React + TypeScript | UI framework |
| Tailwind CSS | Styling |
| `@noble/ed25519` | Client-side keypair generation + transaction signing |
| `@noble/hashes` | Required by noble/ed25519 |
| `bs58` | Base58 encoding (matches Rust bs58 exactly) |
| `zustand` | Global state stores (simulation + wallet) |
| `@tanstack/react-query` | REST data fetching with caching |

---

## 5. Custom Phantom — Identity System

### What it owns

```
src/phantom/

UserDb (Arc<RwLock<UserDbInner>>)
  by_id:       HashMap<user_id, User>
  by_username: HashMap<username, user_id>
  by_email:    HashMap<email, user_id>
  by_google:   HashMap<google_id, user_id>
  by_address:  HashMap<address, user_id>

User {
  id:            String     UUID v4
  username:      String
  email:         String
  password_hash: String     argon2id hash — NEVER plain text
  google_id:     Option<String>
  wallets:       Vec<WalletEntry>
  created_at:    u64        unix ms
}

WalletEntry {
  address:              String   base58 pubkey — the link to Runtime
  encrypted_privkey:    String   AES-256-GCM ciphertext hex
  recovery_phrase:      String   BIP39 12 words — AES-256-GCM ciphertext hex
  kdf_salt:             String   16 random bytes hex — used to derive AES key
  label:                Option<String>
  created_at:           u64
}
```

### What it does — six wallet flows

Every flow produces a `WalletEntry` with an `address`. That address is the only thing the Runtime ever knows about a user.

**Flow 1 — New user, generate wallet**
```
POST /auth/register  { username, password, email, wallet: { mode: "generate" } }

1. generate_recovery_phrase()         → BIP39 12-word phrase
2. generate_keypair(phrase)           → ed25519 SigningKey + VerifyingKey
3. bs58::encode(pubkey)               → address
4. OsRng random 16 bytes              → kdf_salt (hex, stored plaintext)
5. derive_key_from_password(pwd, salt) → 32-byte AES key (NEVER stored)
6. aes_encrypt(aes_key, phrase)       → encrypted_recovery_phrase
7. hash_password(password)            → password_hash (argon2)
8. WalletEntry { address, encrypted_privkey, recovery_phrase, kdf_salt }
9. User::new(username, password_hash, email, wallets: [entry])
10. userdb.insert(user)
11. issue JWT access + refresh tokens
12. return { jwt, address, recovery_phrase }  ← phrase shown ONCE only
```

**Flow 2 — New user, import recovery phrase**
```
POST /auth/register  { ..., wallet: { mode: "import_phrase", phrase: "..." } }

1. validate_phrase(phrase)            → error if invalid BIP39
2. generate_keypair(phrase)           → deterministic keypair
3. bs58::encode(pubkey)               → address
4. userdb.owner_of(address)           → error if already owned by someone
5-12. same as Flow 1 (encrypt phrase, hash password, store, issue JWT)
```

**Flow 3 — New user, import private key**
```
POST /auth/register  { ..., wallet: { mode: "import_privkey", private_key: "base58..." } }

1. is_valid_ed25519_privkey(key)      → validates format
2. bs58::decode(key)                  → privkey_bytes
3. pubkey_from_privkey(privkey_bytes) → pubkey_bytes
4. bs58::encode(pubkey_bytes)         → address
5. userdb.owner_of(address)           → error if already owned
6. fresh kdf_salt
7. derive_key_from_password(pwd, salt) → aes_key
8. aes_encrypt(aes_key, privkey_hex)  → encrypted_private_key
9. WalletSecret::WithPrivateKey { encrypted_private_key }
   (no recovery phrase — privkey import has no phrase)
10-12. hash password, store, issue JWT
```

**Flows 4, 5, 6** — Same as 1, 2, 3 but for an existing logged-in user.
```
POST /user/wallet/new       — flow 4
POST /user/wallet/import    — flows 5 + 6 (mode field selects phrase vs privkey)

Difference: JWT validates user_id first, then userdb.add_wallet(user_id, entry)
```

### What it does NOT own

- Account balances — those are in `AccountsDb` inside the Runtime
- Transaction history — that is in `Vec<Block>` inside the Runtime
- Any simulation state whatsoever

Phantom knows the `address`. It asks the Runtime (via channel) for what the balance and history are.

---

## 6. Mini Solana Runtime — Execution Engine

### What it owns exclusively

```
AccountsDb    HashMap<address, balance>     — current state
Vec<Block>    full block + transaction history
Mempool       VecDeque<Transaction>         — pending queue
PoH chain     running SHA-256 hash          — per leader validator
LeaderScheduler  epoch schedule             — which validator leads each slot
4 Validator tasks
ConsensusEngine task
Slot clock task
NetworkBus    broadcast channel             — simulated P2P
```

### Slot clock

```
tokio::interval(3000ms)
  slot += 1
  if slot % epoch_length == 0:
    scheduler.advance_epoch(new_epoch)
    emit EpochChange event
  leader = scheduler.leader_for_slot(slot)
  network.send(NetworkMessage::SlotTick { slot, epoch, leader })
  emit SlotTick event → WebSocket → frontend
```

### Leader Scheduler — how it works

```
Config: epoch_length=10, stakes = V0:100, V1:200, V2:100, V3:100
Total stake = 500

Slots per epoch per validator:
  V0: (100/500) * 10 = 2 slots
  V1: (200/500) * 10 = 4 slots  ← 2x stake = 2x slots
  V2: (100/500) * 10 = 2 slots
  V3: (100/500) * 10 = 2 slots

Before shuffle: [V0,V0, V1,V1,V1,V1, V2,V2, V3,V3]

LCG deterministic shuffle seeded by epoch number:
  Same epoch → same shuffle result on every machine
  Different epoch → different order, same counts

Epoch 0: [V1,V3,V0,V1,V2,V1,V3,V0,V1,V2]
Epoch 1: [V0,V1,V3,V1,V2,V0,V1,V2,V1,V3]  ← different order, V1 still leads 4/10
```

### Validator — event loop

Each of the 4 validators runs this `tokio::select!` loop forever:

```
select! {
  SlotTick { slot, epoch, leader } →
    if leader == self.id:
      LEADER PATH:
        drain mempool (up to 10 txs)
        acquire write lock on AccountsDb
        BlockBuilder::build():
          poh.advance_slot(slot)       → prev_poh_hash
          for each tx:
            executor.execute(tx, db)
            if success:
              poh.record_transaction(tx.id)
              include tx in block
          state_hash = db.state_hash()
          assemble Block struct
        release write lock
        emit TransactionResult for each tx
        emit BlockProduced event
        consensus_block_tx.send(block.clone())
        network.send(NetworkMessage::NewBlock(block))
    else:
      wait for NewBlock

  NewBlock(block) →
    if block.leader != self.id:
      VALIDATOR PATH:
        Check 1: PohChain::verify_block(block)
          replay sha256 chain from prev_poh_hash
          compare to block.poh_hash
        Check 2: re-execute txs locally
          clone current AccountsDb (read lock)
          execute all txs in block against clone
          compare state hash to block.state_hash
        approved = both checks pass
        emit VoteReceived event
        vote_tx.send(Vote { validator_id, block_slot, approved })

  BlockFinalized { slot } →
    self.finalized_slot = slot
}
```

### Runtime Executor — transaction execution lifecycle

```
Step 1: VERIFY SIGNATURE
  if tx.from != "Faucet":
    decode pubkey from tx.from (base58)
    ed25519_verify(message, signature, pubkey)
    FAIL → return ExecutionResult::failed(InvalidSignature)

Step 2: BASIC VALIDATION
  amount == 0 → ZeroAmount error
  from == to  → SelfTransfer error

Step 3: ENSURE TO ACCOUNT EXISTS
  db.ensure_account(&tx.to)
  creates { address: balance: 0 } if not already present

Step 4: ACQUIRE WRITE LOCKS
  locks.try_acquire_write(&tx.from)
  locks.try_acquire_write(&tx.to)
  either fails → AccountLocked error, release any acquired lock

Step 5: SNAPSHOT
  db.snapshot()    → saves HashMap clone as rollback point

Step 6: EXECUTE TRANSFER
  balance = db.balance(&tx.from)?
  if balance < tx.amount → InsufficientBalance error
  db.debit(&tx.from, tx.amount)
  db.credit(&tx.to, tx.amount)
  compute_units += 550

Step 7: COMPUTE METERING
  if compute_units > 200_000 → ComputeBudgetExceeded error

Step 8: COMMIT OR ROLLBACK
  Ok  → db.commit()    → discard snapshot, keep new state
  Err → db.rollback()  → restore to snapshot (ATOMICALLY)
        note: ensure_account insertion is also rolled back

Step 9: UNLOCK
  locks.release_write(&tx.from)
  locks.release_write(&tx.to)

Step 10: RETURN
  ExecutionResult { tx_id, success, compute_units_used, error }
```

### PoH Chain — how it works

```
Genesis:
  current_hash = sha256("solanalite-genesis-v1")

Every slot:
  prev_poh_hash = sha256("{current_hash}:slot:{slot_number}")
  current_hash  = prev_poh_hash
  (stored as block.prev_poh_hash)

Every successful transaction:
  current_hash = sha256("{current_hash}:tx:{tx_id}")
  (fold the tx ID into the chain)

Final hash after all txs:
  block.poh_hash = current_hash
  (proof of exactly which txs were in this block in this order)

Verification by non-leader validators:
  replay: sha256("{block.prev_poh_hash}:slot:{block.slot}")
  for each tx: sha256("{prev}:tx:{tx.id}")
  final hash must == block.poh_hash
  tampered tx list or wrong order → different final hash → reject
```

### Consensus Engine — 2/3 finalization

```
Tracks votes per slot:
  HashMap<slot, SlotVotes { approvals: HashSet<ValidatorId>, rejections: ... }>

Threshold: (total_validators * 2) / 3 + 1
  With 4 validators: (4*2)/3 + 1 = 3  ← need 3 of 4 approvals

When threshold met:
  1. Acquire write lock on AccountsDb
  2. Re-execute all transactions in block (authoritative commit)
     (validators already executed locally to verify — this is the canonical apply)
  3. Release write lock
  4. chain.write().push(block)   → append to history
  5. network.send(BlockFinalized { slot })
  6. emit SimEvent::BlockFinalized { slot, approvals, total }
  7. emit SimEvent::BalancesUpdated { balances }
  8. votes.remove(slot)   → clean up
```

---

## 7. How the Two Systems Communicate

There are exactly three Tokio channels between Custom Phantom and the Runtime. They never share memory directly. AppState contains only channel senders — no Arc to simulation state.

### Channel 1 — runtime_query_tx (mpsc + oneshot)

**Direction:** Phantom → Runtime (query) / Runtime → Phantom (response)

**Used for:** Reading balance, transaction history, chain data

```
Handler creates oneshot::channel()
Handler sends RuntimeQuery::GetWalletInfo { address, respond_to: tx }
Handler awaits: rx.await
Runtime receives query in select! loop
Runtime reads AccountsDb + scans Vec<Block>
Runtime sends WalletInfo via respond_to.send(info)
Handler returns JSON to frontend

No lock contention — runtime owns the data exclusively
No two handlers competing to read at the same time in a way that causes issues
Clean request/response pattern
```

**RuntimeQuery variants:**
```rust
GetBalance      { address, respond_to: oneshot::Sender<Option<u64>> }
GetWalletInfo   { address, respond_to: oneshot::Sender<WalletInfo> }
GetChain        { respond_to: oneshot::Sender<Vec<Block>> }
GetAllBalances  { respond_to: oneshot::Sender<HashMap<String, u64>> }
```

### Channel 2 — mempool_tx (mpsc)

**Direction:** Phantom → Runtime

**Used for:** Submitting transactions and airdrops into the mempool

```
POST /tx handler:
  validates JWT + ownership
  creates Transaction struct
  mempool_tx.send(tx).await
  returns 200 immediately

Runtime:
  leader validator calls mempool.drain(10) at each slot
  transactions are executed in the next block where this validator is leader
```

### Channel 3 — event_tx (broadcast)

**Direction:** Runtime → Phantom API handlers + WebSocket clients

**Used for:** All live simulation events streamed to frontend

```
Runtime emits events via event_tx.send(SimEvent::...)
Every connected WebSocket client has its own broadcast::Receiver
WS handler loops: events.recv() → socket.send(json)
Frontend receives JSON, updates zustand stores
Components re-render with new data
```

**SimEvent variants and when they fire:**
```
Snapshot          → immediately on WebSocket connect (current state)
SlotTick          → every 3 seconds when slot clock fires
EpochChange       → every 10 slots (epoch boundary)
TransactionQueued → when POST /tx pushes to mempool
BlockProduced     → when leader finishes building block
VoteReceived      → when each validator votes (3 per slot)
BlockFinalized    → when consensus reaches 3/4 threshold
BalancesUpdated   → immediately after finalization (new balances)
TransactionResult → for each tx in the block (success or fail)
```

---

## 8. API Layer — All Routes

### AppState (what handlers receive)

```rust
#[derive(Clone)]
pub struct AppState {
    pub runtime_query_tx: mpsc::Sender<RuntimeQuery>,   // read from runtime
    pub mempool_tx:        mpsc::Sender<Transaction>,   // write to runtime
    pub event_tx:          broadcast::Sender<SimEvent>, // WebSocket stream
    pub users:             Arc<RwLock<UserDb>>,         // phantom user data
}
// NOTE: No AccountsDb, no Vec<Block> here. Runtime owns those.
```

### Auth routes — no JWT required

| Method | Path | Body | Response | What happens |
|---|---|---|---|---|
| POST | `/auth/register` | `{ username, password, email, wallet: { mode, label, phrase?, private_key? } }` | `{ jwt, refresh_token, address, recovery_phrase? }` | Creates user + wallet, returns JWT. Recovery phrase shown once if mode=generate |
| POST | `/auth/login` | `{ username, password }` | `{ access_token, refresh_token }` | Argon2 verify password, issue JWT |
| POST | `/auth/refresh` | `{ refresh_token }` | `{ access_token, refresh_token }` | Issues new token pair |
| POST | `/auth/google/register` | `{ id_token, wallet_password, username, wallet? }` | `{ jwt, address }` | Google JWKS verify, create user |
| POST | `/auth/google/login` | `{ id_token }` | `{ access_token, refresh_token }` | Google JWKS verify, lookup user |

### User routes — JWT required

| Method | Path | Body / Query | Response | What happens |
|---|---|---|---|---|
| GET | `/user/me` | — | `{ user_id, username, email, wallet_count }` | Returns user info from UserDb |
| GET | `/user/wallets` | — | `[{ address, label, created_at }]` | Lists all wallet addresses for this user |
| POST | `/user/wallet/new` | `{ wallet_password, label }` | `{ address, recovery_phrase }` | Generate new keypair, add to user |
| POST | `/user/wallet/import` | `{ mode, wallet_password, label, phrase? or private_key? }` | `{ address }` | Import wallet by phrase or privkey |
| GET | `/user/recovery-phrase` | `?address=&wallet_password=` | `{ phrase }` | Decrypt + return recovery phrase |
| GET | `/user/private-key` | `?address=&wallet_password=` | `{ private_key }` | Decrypt + return private key |
| POST | `/user/sign` | `{ address, wallet_password, message }` | `{ signature }` | Decrypt privkey, sign message, return hex signature |

### Simulation routes — read open, write requires JWT

| Method | Path | Auth | Body | Response | What happens |
|---|---|---|---|---|---|
| POST | `/tx` | JWT | `{ from, to, amount, signature }` | `{ tx_id, queued, message }` | Verify ownership + sig, push to mempool |
| POST | `/airdrop` | JWT | `{ address, amount }` | `{ ok, amount, to }` | System Faucet tx, no sig needed, push to mempool |
| GET | `/balances` | None | — | `{ balances: {addr: balance} }` | Queries Runtime via channel |
| GET | `/chain` | None | — | `[Block]` | Queries Runtime via channel |
| GET | `/validators` | None | — | `[{ id, name, stake }]` | Returns config |
| GET | `/wallet/:address` | JWT | — | `{ address, balance, history }` | Verifies ownership, queries Runtime via channel |
| GET | `/ws` | None | — | WebSocket | Upgrades connection, streams SimEvents |
| GET | `/health` | None | — | `"ok"` | Health check |

### JWT token format

```
Access token:  expires 15 minutes  (JWT_ACCESS_EXPIRY_SECS=900)
Refresh token: expires 7 days      (JWT_REFRESH_EXPIRY_SECS=604800)

Claims: { sub: user_id, exp: timestamp, iat: timestamp }
Header: Authorization: Bearer <access_token>

JWT_SECRET env var required (min 32 chars)
```

---

## 9. Complete Folder Structure

```
solanalite/
├── Cargo.toml                       all backend dependencies
├── .env                             JWT_SECRET, GOOGLE_CLIENT_ID
│
├── src/
│   ├── main.rs                      entry — spawn runtime + start Axum
│   ├── config.rs                    SimConfig, ValidatorConfig
│   ├── types.rs                     Transaction, Block, Vote, NetworkMessage,
│   │                                SimEvent, RuntimeQuery, WalletInfo, TxRecord
│   └── error.rs                     RuntimeError, MempoolError, PhantomError,
│                                    ApiError (with IntoResponse)
│
│   ├── phantom/                     CUSTOM PHANTOM SYSTEM
│   │   ├── mod.rs
│   │   ├── keypair.rs               generate_keypair(phrase) → SigningKey + VerifyingKey
│   │   │                            pubkey_from_privkey(bytes) → pubkey bytes
│   │   │                            is_valid_ed25519_privkey(key) → bool
│   │   │                            is_valid_ed25519_pubkey(key) → bool
│   │   ├── recovery.rs              generate_recovery_phrase() → BIP39 12 words
│   │   │                            validate_phrase(phrase) → Result
│   │   │                            generate_seed(phrase) → [u8;64]
│   │   ├── encrypt.rs               derive_key_from_password(password, salt) → [u8;32]
│   │   │                            aes_encrypt(key, data) → hex_string
│   │   │                            aes_decrypt(key, hex_string) → String
│   │   ├── wallet_service.rs        orchestrates all 6 wallet flows
│   │   │                            unlock_keypair(entry, password) → SigningKey
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── user.rs              User struct, WalletEntry struct, WalletSecret enum
│   │   │   └── store.rs             UserDb — Arc<RwLock<UserDbInner>>
│   │   │                            insert(), add_wallet(), link_google()
│   │   │                            get_by_id/username/email/google()
│   │   │                            owns_address(), owner_of()
│   │   └── auth/
│   │       ├── mod.rs
│   │       ├── password.rs          hash_password(), verify_password() — Argon2id
│   │       ├── jwt.rs               issue_tokens(), verify_token() — jsonwebtoken
│   │       └── google.rs            verify_google_id_token() — JWKS fetch + verify
│   │
│   ├── runtime/                     MINI SOLANA RUNTIME
│   │   ├── mod.rs
│   │   ├── simulator.rs             Simulator::run() — orchestrates all tasks
│   │   │                            handle_query() — responds to RuntimeQuery
│   │   │                            run_slot_clock() — 3s ticker
│   │   ├── poh.rs                   PohChain::new(genesis)
│   │   │                            advance_slot(slot) → prev_poh_hash
│   │   │                            record_transaction(tx_id)
│   │   │                            verify_block(block) → bool
│   │   ├── mempool.rs               Mempool — Arc<Mutex<VecDeque<Transaction>>>
│   │   │                            push() / drain(max) / len() / is_empty()
│   │   ├── network.rs               NetworkBus — broadcast::channel wrapper
│   │   │                            subscribe() / send()
│   │   ├── scheduler.rs             LeaderScheduler
│   │   │                            leader_for_slot(slot) → ValidatorId
│   │   │                            advance_epoch(epoch) → schedule
│   │   │                            is_epoch_boundary(slot) → bool
│   │   ├── consensus.rs             ConsensusEngine
│   │   │                            run(vote_rx, block_rx)
│   │   │                            handle_vote() — track + threshold check
│   │   │                            finalize() — commit + broadcast
│   │   ├── accounts/
│   │   │   ├── mod.rs
│   │   │   └── db.rs                AccountsDb
│   │   │                            new(initial_balances)
│   │   │                            balance() / all_balances() / exists()
│   │   │                            ensure_account() / debit() / credit()
│   │   │                            snapshot() / commit() / rollback()
│   │   │                            state_hash() — sha256 of sorted pairs
│   │   ├── executor/
│   │   │   ├── mod.rs
│   │   │   ├── executor.rs          RuntimeExecutor
│   │   │   │                        execute(tx, db) → ExecutionResult
│   │   │   │                        verify_signature() — ed25519 verify
│   │   │   │                        AccountLocks — HashSet<address>
│   │   │   └── builder.rs           BlockBuilder
│   │   │                            build(slot, epoch, leader, txs, poh, db)
│   │   │                              → (Block, Vec<ExecutionResult>)
│   │   └── validator/
│   │       ├── mod.rs               Validator
│   │       │                        run(net_rx) — select! event loop
│   │       │                        on_leader_slot() — produce block
│   │       │                        on_received_block() — validate + vote
│   │       │                        validate_block() — poh + state hash check
│   │       └── state.rs             ValidatorState enum
│   │
│   └── api/
│       ├── mod.rs
│       ├── router.rs                build_router(state) → Router
│       │                            mounts phantom/ + simulation/ + CORS
│       ├── state.rs                 AppState — channels + UserDb only
│       ├── ws.rs                    ws_handler — upgrade + snapshot + stream
│       ├── middleware/
│       │   └── auth.rs              JwtClaims extractor — FromRequestParts
│       └── handlers/
│           ├── mod.rs
│           ├── phantom/
│           │   ├── mod.rs
│           │   ├── register.rs      POST /auth/register
│           │   ├── login.rs         POST /auth/login
│           │   ├── refresh.rs       POST /auth/refresh
│           │   ├── google_register.rs POST /auth/google/register
│           │   ├── google_login.rs  POST /auth/google/login
│           │   ├── wallets.rs       GET  /user/wallets
│           │   ├── wallet_new.rs    POST /user/wallet/new
│           │   ├── wallet_import.rs POST /user/wallet/import
│           │   ├── recovery.rs      GET  /user/recovery-phrase
│           │   ├── privkey.rs       GET  /user/private-key
│           │   └── sign.rs          POST /user/sign
│           └── simulation/
│               ├── mod.rs
│               ├── tx.rs            POST /tx
│               ├── airdrop.rs       POST /airdrop
│               ├── balances.rs      GET  /balances
│               ├── chain.rs         GET  /chain
│               ├── wallet_info.rs   GET  /wallet/:address
│               └── validators.rs    GET  /validators
│
└── frontend/
    ├── package.json
    ├── vite.config.ts               proxy /tx /balances /chain /validators /airdrop → :3001
    ├── index.html
    └── src/
        ├── main.tsx                 ReactDOM.createRoot entry
        ├── App.tsx                  root layout — composes all panels
        ├── types/
        │   └── sim.ts               mirrors all Rust types (SimEvent, Block, Transaction...)
        ├── store/
        │   ├── simulation.ts        zustand — slot, epoch, leader, balances, blocks, events
        │   └── wallet.ts            zustand — user session, wallet list, current address
        ├── hooks/
        │   ├── useWebSocket.ts      connects to ws://localhost:3001/ws, feeds stores
        │   └── useWalletInfo.ts     react-query GET /wallet/:address
        ├── lib/
        │   ├── wallet.ts            createWallet(), signTransaction() — @noble/ed25519
        │   └── api.ts               typed fetch wrappers for all routes
        └── components/
            ├── Onboarding.tsx       register, login, create wallet flow
            ├── WalletPanel.tsx      address, balance, tx history
            ├── SlotHeader.tsx       slot · epoch · leader · connection badge
            ├── ValidatorCard.tsx    id · stake · state · last vote · leader glow
            ├── BalanceTable.tsx     account → balance · animated ↑↓ delta
            ├── BlockFeed.tsx        scrollable finalized block list
            ├── EventLog.tsx         monospace terminal stream
            └── TransactionForm.tsx  from/to dropdowns · amount · sign + POST /tx
```

---

## 10. Every Data Structure

### Simulation types (`src/types.rs`)

```rust
pub type ValidatorId = u32;
pub type Pubkey      = String;
pub type SlotNumber  = u64;

pub struct Transaction {
    pub id:           String,    // UUID v4
    pub from:         Pubkey,    // base58 address
    pub to:           Pubkey,    // base58 address
    pub amount:       u64,       // in lamports
    pub signature:    String,    // hex-encoded ed25519 sig (empty for Faucet)
    pub submitted_at: u64,       // unix ms
    pub slot:         u64,       // filled by leader when included in block
    pub success:      bool,      // filled by executor
    pub error:        Option<String>, // filled by executor on failure
}

pub struct Block {
    pub slot:          SlotNumber,
    pub epoch:         u64,
    pub leader:        ValidatorId,
    pub transactions:  Vec<Transaction>,  // only SUCCESSFUL txs
    pub prev_poh_hash: String,  // hash before any txs this slot
    pub poh_hash:      String,  // hash after all successful txs
    pub state_hash:    String,  // sha256(sorted address:balance pairs)
    pub produced_at:   u64,     // unix ms
}

pub struct Vote {
    pub validator_id: ValidatorId,
    pub block_slot:   SlotNumber,
    pub approved:     bool,
    pub reason:       Option<String>,
}

pub enum NetworkMessage {
    SlotTick { slot: SlotNumber, epoch: u64, leader: ValidatorId },
    NewBlock(Block),
    BlockFinalized { slot: SlotNumber },
}

pub enum RuntimeQuery {
    GetBalance    { address: String, respond_to: oneshot::Sender<Option<u64>> },
    GetWalletInfo { address: String, respond_to: oneshot::Sender<WalletInfo> },
    GetChain      { respond_to: oneshot::Sender<Vec<Block>> },
    GetAllBalances{ respond_to: oneshot::Sender<HashMap<String, u64>> },
}

pub struct WalletInfo {
    pub address: String,
    pub balance: u64,
    pub history: Vec<TxRecord>,
}

pub struct TxRecord {
    pub tx_id:        String,
    pub direction:    String,   // "sent" or "received"
    pub counterparty: String,   // the other address
    pub amount:       u64,
    pub slot:         u64,
    pub success:      bool,
    pub timestamp:    u64,
}

pub enum SimEvent {
    Snapshot         { balances: HashMap<String,u64>, slot: u64 },
    SlotTick         { slot: u64, epoch: u64, leader: u32, leader_name: String },
    EpochChange      { epoch: u64, schedule: Vec<(u64, u32)> },
    TransactionQueued{ tx_id: String, from: String, to: String, amount: u64 },
    BlockProduced    { slot: u64, leader: u32, tx_count: usize, poh_hash: String, state_hash: String },
    VoteReceived     { validator: u32, validator_name: String, slot: u64, approved: bool, reason: Option<String> },
    BlockFinalized   { slot: u64, approvals: usize, total: usize },
    BalancesUpdated  { balances: HashMap<String,u64> },
    TransactionResult{ tx_id: String, success: bool, reason: Option<String> },
}
```

### Phantom types (`src/phantom/db/`)

```rust
pub struct User {
    pub id:            String,
    pub username:      String,
    pub email:         String,
    pub password_hash: String,
    pub google_id:     Option<String>,
    pub wallets:       Vec<WalletEntry>,
    pub created_at:    u64,
}

pub struct WalletEntry {
    pub address:           String,   // base58 pubkey — the bridge to Runtime
    pub secret:            WalletSecret,
    pub kdf_salt:          String,   // hex 16 bytes — stored plaintext
    pub label:             Option<String>,
    pub created_at:        u64,
}

pub enum WalletSecret {
    WithPhrase    { encrypted_recovery_phrase: String },
    WithPrivateKey{ encrypted_private_key:     String },
}
```

### Config (`src/config.rs`)

```rust
pub struct SimConfig {
    pub epoch_length:      u64,    // 10
    pub slot_duration_ms:  u64,    // 3000
    pub genesis_hash:      String, // "solanalite-genesis-v1"
    pub validators:        Vec<ValidatorConfig>,
    pub initial_balances:  Vec<(String, u64)>,
    // initial_balances includes ("Faucet", 1_000_000) for airdrops
}

pub struct ValidatorConfig {
    pub id:    u32,
    pub name:  String,
    pub stake: u64,
}
// V0: 100, V1: 200, V2: 100, V3: 100
```

---

## 11. Step by Step — User Journeys

### Journey 1: New user, first time

```
1. User opens browser → Onboarding.tsx shows register form
2. User enters username, password, email → clicks "Create Wallet"
3. Frontend: POST /auth/register { username, password, email, wallet: { mode: "generate" } }
4. Backend:
   a. generate_recovery_phrase()         12 BIP39 words
   b. generate_keypair(phrase)           ed25519 keypair
   c. address = bs58(pubkey)
   d. kdf_salt = random 16 bytes
   e. aes_key  = derive_key_from_password(password, kdf_salt)  ← NOT stored
   f. encrypted_phrase = aes_encrypt(aes_key, phrase)
   g. password_hash = argon2id_hash(password)
   h. WalletEntry { address, WithPhrase { encrypted_phrase }, kdf_salt }
   i. User { username, password_hash, email, wallets: [entry] }
   j. userdb.insert(user)
   k. issue JWT access + refresh tokens
5. Response: { access_token, refresh_token, address, recovery_phrase: "word1 word2..." }
6. Frontend: store JWT in zustand wallet store
7. Frontend: SHOW RECOVERY PHRASE TO USER — they must save it
   (server will NEVER show it again without password confirmation)
8. User is logged in, address displayed in WalletPanel
```

### Journey 2: Existing user logs in

```
1. POST /auth/login { username, password }
2. userdb.get_by_username(username)
3. argon2id_verify(password, user.password_hash)  — FAIL → 401 Unauthorized
4. PASS → issue new JWT access + refresh tokens
5. Response: { access_token, refresh_token }
6. Frontend stores tokens, loads wallet list via GET /user/wallets
```

### Journey 3: User requests airdrop

```
1. User clicks "Request Airdrop 100 LAM"
2. Frontend: POST /airdrop { address: "4Nd1m...", amount: 100 }
   Authorization: Bearer <access_token>
3. Handler:
   a. JWT middleware extracts user_id
   b. verify user owns address via UserDb
   c. cap amount at 100
   d. Transaction { from: "Faucet", to: address, amount: 100, signature: "system" }
   e. mempool_tx.send(tx)   → crosses channel boundary to Runtime
4. Response: 200 { ok: true, amount: 100 }
5. Frontend shows "pending" indicator
6. Runtime (next slot clock tick):
   a. current leader drains mempool
   b. executor.execute(tx, db):
      - tx.from == "Faucet" → skip signature check
      - Faucet balance -= 100
      - user address balance += 100 (or ensure_account + credit)
   c. block produced, votes collected, block finalized
   d. event_tx broadcasts BalancesUpdated
7. WebSocket delivers BalancesUpdated to frontend
8. WalletPanel balance updates: 0 → 100 LAM
```

### Journey 4: User sends a transaction

```
1. User opens TransactionForm: From = their address, To = another address, Amount = 10
2. Frontend:
   a. message = `${tx_id}:${from}:${to}:${amount}`
   b. signature = await ed.signAsync(message_bytes, secretKey)
   c. POST /tx { from, to, amount, signature: hex(signature) }
      Authorization: Bearer <access_token>
3. Handler:
   a. JWT middleware — extracts user_id
   b. Validate: from not empty, to not empty, from != to, amount > 0, signature not empty
   c. bs58::decode(from) — valid address format check
   d. state.users.owns_address(user_id, from) — user must own the from address
   e. Transaction { id: UUID, from, to, amount, signature, submitted_at }
   f. emit SimEvent::TransactionQueued
   g. mempool_tx.send(tx)   → Runtime receives
   h. Response: 200 { tx_id, queued: true }
4. Frontend: shows tx as "pending" in EventLog
5. Runtime:
   a. leader drains mempool next slot
   b. executor.execute():
      - verify_signature(tx) — ed25519 decode pubkey from from address, verify sig
      - check balance
      - debit from, credit to
      - record tx in PoH
   c. emit TransactionResult { tx_id, success: true }
   d. consensus finalizes, emit BalancesUpdated
6. Frontend: EventLog shows confirmed, BalanceTable animates
```

---

## 12. Step by Step — Transaction Lifecycle

This is the life of one transaction from POST to finalized balance.

```
POST /tx arrives at Axum
  ↓
api/handlers/simulation/tx.rs
  validate JWT ← middleware
  validate body fields
  validate address format
  verify user owns "from" address
  Transaction::new(from, to, amount, signature)
  emit TransactionQueued event
  mempool_tx.send(tx)          ← mpsc to Runtime
  return { tx_id, queued: true }
  ↓
WebSocket → frontend
  { type: "TransactionQueued", data: { tx_id, from, to, amount } }
  frontend shows "pending" badge
  ↓
tx sits in mempool VecDeque
  ↓
3 seconds pass → slot clock ticks
  ↓
NetworkMessage::SlotTick broadcast to all 4 validators
  ↓
scheduler.leader_for_slot(slot) = ValidatorId X
Validator X receives SlotTick, sees it is the leader
  ↓
mempool.drain(10)
  pulls transaction out
  ↓
accounts.write().await  ← exclusive write lock on AccountsDb
BlockBuilder::build():
  poh.advance_slot(slot)         → prev_poh_hash saved
  executor.execute(tx, db):
    verify_signature()           ← decode pubkey from "from" address
    ensure_account(tx.to)        ← create if first time receiving
    lock "from" + "to" accounts
    db.snapshot()
    check balance
    db.debit("from", amount)
    db.credit("to", amount)
    db.commit()
    unlock accounts
    return ExecutionResult { success: true }
  poh.record_transaction(tx.id)  ← fold into chain
  include tx in block.transactions
  state_hash = db.state_hash()
  Block { slot, epoch, leader, transactions, prev_poh_hash, poh_hash, state_hash }
accounts write lock released
  ↓
emit TransactionResult { tx_id, success: true }   → WebSocket → frontend
emit BlockProduced { slot, leader, tx_count, poh_hash, state_hash }
consensus_block_tx.send(block.clone())   ← consensus engine receives block
network.send(NetworkMessage::NewBlock(block))  ← broadcast to other validators
  ↓
WebSocket → frontend
  { type: "BlockProduced", data: { slot, leader, tx_count } }
  ↓
V0, V2, V3 receive NewBlock
Each independently:
  PohChain::verify_block():
    replay sha256 chain from prev_poh_hash
    check final == block.poh_hash  ✓
  clone AccountsDb (read lock)
  re-execute transactions against clone
  compare state hash  ✓
  vote_tx.send(Vote { validator_id, block_slot, approved: true })
  emit VoteReceived event
  ↓
WebSocket → frontend
  { type: "VoteReceived", data: { validator: 0, approved: true } }  × 3
  ↓
ConsensusEngine collects votes
  slot X: approvals = 1  → not yet (need 3)
  slot X: approvals = 2  → not yet
  slot X: approvals = 3  → THRESHOLD MET
  ↓
finalize(slot):
  accounts.write().await
  RuntimeExecutor re-executes block txs  ← authoritative commit
  db.commit()
  accounts write lock released
  chain.write().push(block)
  network.send(BlockFinalized { slot })  → all validators update finalized_slot
  emit BlockFinalized { slot, approvals: 3, total: 4 }
  emit BalancesUpdated { balances: { "from": X-10, "to": Y+10 } }
  ↓
WebSocket → frontend
  { type: "BlockFinalized", data: { slot, approvals: 3, total: 4 } }
  { type: "BalancesUpdated", data: { balances: {...} } }
  ↓
Frontend:
  block card shows "FINALIZED ✓"
  BalanceTable animates: "from" ↓ -10, "to" ↑ +10
  WalletPanel balance updates
  tx shows "confirmed"
  ↓
Total time: approximately 3–6 seconds (1–2 slot durations)
```

---

## 13. Data Security Model

### The fundamental principle

**The server never stores a private key in plaintext. Ever.**
**The server never sees a private key during any normal operation.**
**Ownership is proven per-transaction by Ed25519 signature. No sessions needed.**

### Cryptographic model — stored data

```
What is stored in UserDb:
  password_hash       argon2id hash of user password
                      cannot be reversed to get plain password
  kdf_salt            16 random bytes hex — NOT secret
                      only useful for deriving AES key (need password for that)
  encrypted_privkey   AES-256-GCM ciphertext of the private key bytes
                      useless without the AES key
                      AES key useless without the user's plain password
  encrypted_phrase    AES-256-GCM ciphertext of the BIP39 recovery phrase
                      same protection as encrypted_privkey

What is NEVER stored:
  plain password
  AES encryption key    — derived fresh each time from password + kdf_salt
  plain private key     — decrypted in-memory on demand, immediately dropped
  plain recovery phrase — encrypted before storage, shown once at creation
```

### AES key derivation

```
Derive AES key from password:
  input:  raw password (user types it)
          kdf_salt (stored plaintext in WalletEntry)
  process: Argon2id KDF mode (NOT verification mode)
           outputs 32 bytes
  output: 32-byte AES-256 key
  stored: NOWHERE — derived fresh on each operation

This means:
  To read private key → need password → derive AES key → decrypt
  Database breach gives only ciphertext → useless without password
```

### AES encryption format

```
Encrypt:
  1. Generate fresh 12-byte nonce with OsRng (random, single use)
  2. AES-256-GCM encrypt(key, nonce, plaintext) → ciphertext
  3. Store as hex(nonce || ciphertext)

Decrypt:
  1. hex_decode(stored_string) → bytes
  2. Split: first 12 bytes = nonce, rest = ciphertext
  3. AES-256-GCM decrypt(key, nonce, ciphertext) → plaintext

Fresh nonce on every encryption means:
  Same data encrypted twice = different ciphertext each time
  No nonce reuse vulnerability
```

### Ed25519 signature model

```
What travels over the network for a transaction:
  from:      base58 address (= public key encoded)
  to:        base58 address
  amount:    u64
  signature: hex-encoded ed25519 signature

What the backend does with this:
  1. Decode pubkey from from address (base58 decode)
  2. Reconstruct message = "{tx_id}:{from}:{to}:{amount}"
  3. ed25519_verify(message_bytes, signature_bytes, pubkey)
  4. PASS → allowed to debit "from" account
  5. FAIL → reject immediately

What the backend does NOT do:
  Look up a private key
  Look up a password
  Check a session token
  Consult UserDb at all

The math proves ownership. The database is not involved.
```

### What a database breach exposes

```
Exposed:
  argon2id password hashes → computationally expensive to crack
  kdf_salts → not secret by design
  AES-256-GCM ciphertexts → useless without password
  wallet addresses → public by design (anyone can see on-chain)
  usernames, emails → social data only

NOT exposed:
  any private key
  any recovery phrase
  any AES key
  any plain password
  any session secret
```

### JWT security

```
JWT_SECRET env var — minimum 32 characters
Stored only in env, never in code or database
access_token expires in 15 minutes
refresh_token expires in 7 days

JWT is used to verify:
  Who you are (user_id in claims)
  Whether you are allowed to submit a transaction (own the "from" address)

JWT is NOT used to derive encryption keys (separate wallet_password for that)
Google auth uses google_id for identity only — never for key derivation
```

---

## 14. Error Handling

### Error types and where they live

```rust
// error.rs

PhantomError        // user system failures
  UserNotFound(String)
  UsernameTaken(String)
  EmailTaken(String)
  AddressAlreadyExists(String)
  GoogleAlreadyLinked
  InvalidPassword
  WalletNotFound(String)
  Unauthorized

RuntimeError        // simulation execution failures
  AccountNotFound(String)
  InsufficientBalance { account, have, need }
  ZeroAmount
  SelfTransfer
  AccountLocked(String)
  ComputeBudgetExceeded
  InvalidSignature
  MissingSignature
  InvalidAddress

MempoolError        // mempool failures
  Full(usize)

ApiError            // HTTP layer — implements IntoResponse
  MissingField(String)     → 400
  InvalidAddress(String)   → 400
  InvalidAmount(String)    → 400
  SelfTransfer             → 400
  MissingSignature         → 400
  AirdropLimitExceeded     → 422
  Unauthorized             → 401
  NotFound(String)         → 404
  Conflict(String)         → 409
  MempoolFull              → 503
  Internal(String)         → 500
```

### Error conversion chain

```
PhantomError → ApiError   (From impl)
MempoolError → ApiError   (From impl)
RuntimeError → ApiError   (From impl — only for sync errors at API boundary)

Handler return type: Result<Json<T>, ApiError>
ApiError::into_response() → (StatusCode, Json { ok: false, code: "...", error: "..." })
? operator converts automatically
```

### What frontend receives

```json
{ "ok": false, "code": "MISSING_SIGNATURE", "error": "missing signature" }
{ "ok": false, "code": "INSUFFICIENT_BALANCE", "error": "alice has 5, needs 100" }
{ "ok": false, "code": "MEMPOOL_FULL", "error": "mempool is full, try again later" }
{ "ok": false, "code": "UNAUTHORIZED", "error": "not authorized" }
```

Frontend switches on `code` (stable machine-readable) not `error` (human readable string that can change).

### Async errors via WebSocket

Most execution errors are async — the HTTP response returns `200 queued:true` immediately. The actual result comes via WebSocket:

```json
{ "type": "TransactionResult", "data": { "tx_id": "abc", "success": false, "reason": "insufficient balance: alice has 5, needs 100" } }
```

---

## 15. Frontend Architecture

### State management — two zustand stores

**simulation.ts** — all runtime state

```typescript
interface SimStore {
  connected:    boolean
  slot:         number
  epoch:        number
  leader:       number
  balances:     Record<string, number>
  prevBalances: Record<string, number>  // for delta animation
  validators:   ValidatorDisplay[]
  blocks:       BlockDisplay[]
  events:       SimEvent[]              // last 150 events
  applyEvent:   (event: SimEvent) => void
}
```

**wallet.ts** — user session state

```typescript
interface WalletStore {
  accessToken:  string | null
  refreshToken: string | null
  userId:       string | null
  username:     string | null
  wallets:      WalletEntry[]
  activeAddress:string | null
  secretKey:    Uint8Array | null   // in memory only, never persisted
  setUser:      (user) => void
  setSecretKey: (key) => void
  logout:       () => void
}
```

### WebSocket hook

```
useWebSocket.ts connects to ws://localhost:3001/ws
On connect: receive Snapshot → populate simulation store
On message: parse SimEvent → store.applyEvent(event)
On disconnect: auto-reconnect after 2000ms
Components subscribe to only the slice they need:
  SlotHeader:   useSimStore(s => ({ slot: s.slot, epoch: s.epoch, leader: s.leader }))
  BalanceTable: useSimStore(s => s.balances)
  ValidatorCard:useSimStore(s => s.validators.find(v => v.id === id))
```

### Client-side signing

```typescript
// lib/wallet.ts — keys NEVER leave the browser
import * as ed from '@noble/ed25519'
import bs58 from 'bs58'

export async function createWallet() {
  const secretKey = ed.utils.randomPrivateKey()     // 32 random bytes
  const publicKey = await ed.getPublicKeyAsync(secretKey)
  const address   = bs58.encode(publicKey)          // matches Rust bs58
  return { secretKey, publicKey, address }
  // secretKey stored only in zustand wallet store (memory)
  // never sent to server, never written to localStorage
}

export async function signTransaction(tx, secretKey) {
  const message = `${tx.id}:${tx.from}:${tx.to}:${tx.amount}`
  const bytes   = new TextEncoder().encode(message)
  const sig     = await ed.signAsync(bytes, secretKey)
  return toHex(sig)
  // Only the hex signature travels to the server
}
```

### Proxy config (vite.config.ts)

```typescript
server: {
  proxy: {
    '/tx':         'http://localhost:3001',
    '/airdrop':    'http://localhost:3001',
    '/balances':   'http://localhost:3001',
    '/chain':      'http://localhost:3001',
    '/validators': 'http://localhost:3001',
    '/wallet':     'http://localhost:3001',
    '/auth':       'http://localhost:3001',
    '/user':       'http://localhost:3001',
  }
}
// WebSocket connects directly: ws://localhost:3001/ws (no proxy needed)
```

---

## 16. Build Order

Build each stage completely before starting the next. Every stage ends with something that compiles and is testable.

```
Stage 1  — Project skeleton
           Cargo.toml, empty module files, mod declarations
           Verify: cargo build passes

Stage 2  — Domain types
           config.rs, types.rs, error.rs
           Verify: cargo check passes

Stage 3  — AccountsDb
           accounts/db.rs
           Tests: balance, debit, credit, snapshot/rollback, state_hash
           Verify: cargo test accounts (7 tests pass)

Stage 4  — PoH Chain
           poh.rs
           Tests: advance, deterministic, verify empty/with-tx, tampered-fails
           Verify: cargo test poh (5 tests pass)

Stage 5  — Runtime Executor
           runtime/executor.rs
           Tests: success, rollback, zero-amount, self-transfer, new-account, not-found
           Verify: cargo test runtime::executor (6 tests pass)

Stage 6  — Block Builder
           runtime/builder.rs
           Tests: empty block, success, failed-not-included, deterministic-hash
           Verify: cargo test runtime::builder (4 tests pass)

Stage 7  — Mempool
           mempool.rs
           Tests: push+drain, capacity, drain-more-than-available
           Verify: cargo test mempool (3 tests pass)

Stage 8  — Leader Scheduler
           scheduler.rs
           Tests: equal-stakes, stake-weighted, same-epoch-same-schedule,
                  epoch-boundary, different-epochs-different-schedules
           Verify: cargo test scheduler (5 tests pass)

Stage 9  — Network Bus
           network.rs
           Verify: cargo check

Stage 10 — Slot Clock
           simulator.rs (clock function only)
           Verify: run with temp main.rs, see slot ticks in terminal

Stage 11 — Single Validator
           validator/mod.rs + validator/state.rs
           Verify: run with 1 validator, see it produce blocks when leader

Stage 12 — Four Validators + Orchestrator
           simulator.rs (full run() function)
           Verify: cargo run, see blocks produced, votes, no panics

Stage 13 — Consensus Engine
           consensus.rs
           Verify: see "Block X FINALIZED" in terminal after 3 votes

Stage 14 — Axum API
           api/router.rs, api/state.rs, api/handlers/simulation/
           Verify: curl /health, /balances, POST /tx, /chain

Stage 15 — WebSocket
           api/ws.rs
           Verify: wscat -c ws://localhost:3001/ws → snapshot + live events

Stage 16 — Phantom user system
           phantom/ (all files)
           api/handlers/phantom/
           api/middleware/auth.rs
           Verify: POST /auth/register, POST /auth/login, GET /user/wallets

Stage 17 — Frontend
           frontend/ (Bun + Vite + React)
           Build in order: useWebSocket → SlotHeader → ValidatorCards →
           BalanceTable → BlockFeed → EventLog → TransactionForm →
           Onboarding → WalletPanel
           Verify: browser shows live dashboard
```

---

## 17. Running the Project

### Prerequisites

```bash
# Rust (stable >= 1.75)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Bun
curl -fsSL https://bun.sh/install | bash

# Optional dev tools
cargo install cargo-watch
npm install -g wscat
```

### Environment variables

```bash
# backend/.env
JWT_SECRET=your-secret-at-least-32-characters-long
GOOGLE_CLIENT_ID=your-google-oauth-client-id   # optional
JWT_ACCESS_EXPIRY_SECS=900                      # optional, default 900
JWT_REFRESH_EXPIRY_SECS=604800                  # optional, default 604800
```

### Start everything

```bash
# Terminal 1 — Rust backend
cd solanalite
cargo run
# Server: http://localhost:3001
# Simulation starts immediately

# Terminal 2 — Bun frontend
cd solanalite/frontend
bun install
bun run dev
# Frontend: http://localhost:5173

# Open browser
http://localhost:5173
```

### Test the API directly

```bash
# Health
curl http://localhost:3001/health

# Register + generate wallet
curl -X POST http://localhost:3001/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123","email":"a@a.com","wallet":{"mode":"generate","label":"main"}}'

# Login
curl -X POST http://localhost:3001/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123"}'

# Airdrop (use token from login)
curl -X POST http://localhost:3001/airdrop \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <access_token>" \
  -d '{"address":"<your_address>","amount":100}'

# All balances
curl http://localhost:3001/balances

# Block history
curl http://localhost:3001/chain

# WebSocket
wscat -c ws://localhost:3001/ws
```

---

## 18. Key Design Decisions Explained

### Why two separate systems in one binary?

Separation of concerns. Custom Phantom handles *identity* (who you are), Mini Solana Runtime handles *execution* (what happens). They communicate via channels so neither system can accidentally corrupt the other's state. If someone breaks the auth logic, it cannot corrupt blockchain state. If the simulation panics, it does not affect user data.

### Why channels instead of shared Arc for cross-system communication?

If AppState held `Arc<RwLock<AccountsDb>>` directly, handlers would compete for the same RwLock as the consensus engine during finalization. Using channels means the Runtime is the single owner of its state. Handlers send queries and wait for responses. No lock contention between HTTP handlers and the simulation loop.

### Why does the backend never store the private key?

Because it does not need to. The backend only needs to verify that a transaction was signed by the owner of the `from` address. The `from` address *is* the public key (base58 encoded). So the backend decodes the public key from the address and uses it to verify the signature. It never needs the private key for this. The private key stays in the browser.

### Why is ensure_account called only for tx.to?

Because `tx.from` must already exist with sufficient funds — if it does not exist, the balance check will fail and the transaction is rejected (the executor returns InsufficientBalance). Auto-creating the sender with balance 0 would just immediately fail the same check, so there is no point. The recipient however may be a new address receiving funds for the first time — creating it with balance 0 before crediting is correct.

### Why does consensus re-execute transactions to commit?

Validators execute locally to *verify* the block (they need the state hash to vote). The consensus engine executes the transactions again to *authoritatively commit* to shared AccountsDb. This is the canonical, trusted write. All local executions before this are just verification. This ensures exactly one authoritative commit to shared state regardless of how many validators ran the txs locally.

### Why is the PoH chain important?

Proof of History solves the ordering problem without trusting a central clock. Every slot and every transaction is folded into the running hash. If you know the hash before a block and the hash after, you can prove exactly which transactions happened in exactly which order, and that nothing was inserted or removed. Validators replay the hash to verify this. It is impossible to produce the correct final hash without having processed the exact same transactions in the exact same order.

### Why does V1 lead more slots?

Stake-weighted leadership. V1 has stake 200, the others have 100 each. V1's proportional share is 200/500 = 40% of slots. In an epoch of 10 slots, V1 leads 4, each other validator leads 2. This mirrors real Solana where validators with more stake (more SOL delegated) earn proportionally more leadership slots — and thus more transaction fees.

---

## 19. How to Answer Judge Questions

**"What is PoH?"**

Proof of History is a cryptographic hash chain that creates a verifiable record of time and ordering. Every slot advances the chain with `sha256(prev_hash:slot:N)`. Every successful transaction is folded in with `sha256(prev_hash:tx:tx_id)`. The final hash in a block is proof that exactly those transactions happened in exactly that order. Validators replay the hash operations from `prev_poh_hash` to verify a block without trusting the leader.

**"Why does V1 appear more in the leader schedule?"**

V1 has stake 200 versus 100 for each other validator. Total stake is 500. V1's proportional share is 200/500 = 40%. In an epoch of 10 slots, V1 leads 4 slots. This is stake-weighted leader scheduling — validators with more stake earn proportionally more leadership slots, which mirrors real Solana's incentive structure.

**"How do you prevent someone from stealing Alice's funds?"**

They would need to produce a valid Ed25519 signature over the transaction data using Alice's private key. Without Alice's private key, this is computationally infeasible. The signature cannot be forged. The backend verifies the signature on every transaction by decoding the public key from the sender's address (the address IS the public key) and running `ed25519_verify`. No private key is stored on the server.

**"What happens if a validator sends a bad block?"**

Other validators independently re-execute all transactions against their local snapshot of AccountsDb and compare the resulting state hash to `block.state_hash`. If the hashes differ, they vote rejected. They also replay the PoH hash chain from `block.prev_poh_hash`. If `PohChain::verify_block` returns false, they vote rejected. With 4 validators, if 2 or more reject, approvals stay below the threshold of 3 and the block never finalizes.

**"Where is the private key stored?"**

For users who generate a wallet: the private key is derived on demand from the encrypted recovery phrase. The recovery phrase is stored AES-256-GCM encrypted in the database. The AES key is never stored — it is derived fresh from the user's password using Argon2id KDF whenever needed. For users who import a raw private key: the private key bytes are AES-256-GCM encrypted with the same mechanism. A database breach exposes only argon2 hashes and AES ciphertexts — useless without the user's password.

**"Why do you re-execute in consensus if validators already executed?"**

Validators execute locally against a *snapshot* for verification purposes only — they need the state hash to vote. The consensus engine performs the single authoritative commit to the shared `AccountsDb` when the threshold is reached. This ensures there is exactly one canonical write to shared state, performed only after 2/3 agreement, not speculatively during validation.

**"What is the difference between Custom Phantom and the Runtime?"**

Custom Phantom handles *identity* — who you are, what keypairs you own, proving you are authorized to act. It owns UserDb. Mini Solana Runtime handles *execution* — what actually happens to balances and what gets recorded in blocks. It owns AccountsDb and Vec<Block>. They communicate via three Tokio channels and never touch each other's state directly. The only link between them is the wallet address string.

---

*End of Solite complete documentation.*