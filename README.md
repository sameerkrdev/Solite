# Mini Solana Validator Runtime

A single-binary Rust simulator that mirrors the internal architecture of a Solana validator cluster — complete with proof-of-history, stake-weighted leader scheduling, parallel transaction execution, 2/3 supermajority consensus, and a live React dashboard.

---

## What it is

This project simulates how a real Solana validator cluster works, running **4 validators as concurrent async tasks**. Every 3 seconds a slot ticks, a leader is elected, transactions are executed, a block is produced, and the network reaches consensus — all observable in real time through a WebSocket-powered frontend.

It is **not** a real Solana node. There is no P2P network, no real VDF-based PoH, and no token economics. It is an architectural simulator that faithfully mirrors the concepts — built to demonstrate how the pieces fit together.

---

## What it demonstrates

- **Validator lifecycle** — each validator runs its own async event loop, switching between idle, producing, validating, and voting states
- **Proof of History** — a SHA-256 hash chain links every slot and every transaction in order; validators replay the chain to verify blocks
- **Stake-weighted leader scheduling** — slots are distributed proportionally by stake, shuffled deterministically per epoch using a seeded LCG
- **Epoch rotation** — every 10 slots a new leader schedule is computed; the validator with 2× stake gets 2× leader slots
- **Parallel transaction execution** — account-level write locking, snapshot/rollback for atomicity, compute unit metering
- **2/3 supermajority consensus** — 3 of 4 validators must approve a block for it to finalize; rejection below threshold leaves the block pending
- **Live observability** — every event streams to the React frontend via WebSocket in real time

---

## Stack

| Layer | Technology |
|---|---|
| Backend runtime | Rust (stable ≥ 1.75, edition 2021) |
| Async runtime | Tokio (full features) |
| HTTP + WebSocket | Axum 0.7 |
| Serialization | Serde + serde_json |
| Hashing | SHA-2 + hex |
| Error handling | thiserror + anyhow |
| Frontend | React + TypeScript (Vite) |
| Styling | Tailwind CSS |
| Package manager | Bun |

---

## Architecture overview

```
React Frontend (Vite + Bun)
        │ WebSocket (push events)     │ HTTP REST (POST /tx, GET /balances)
        ▼                             ▼
    Axum Server — port 3001
        │                  │
   event broadcast      mempool mpsc
        │                  │
        └──────── Simulation Core ────────────┐
                           │                  │
                    Slot Clock (3s)     Consensus Engine
                           │            (collects votes,
                    Network Bus          fires finalization)
                (broadcast channel)
                    │   │   │   │
                   V0  V1  V2  V3   ← 4 concurrent async validator tasks
                        │
                   (when leader)
                  BlockBuilder
                  RuntimeExecutor
                  PohChain
```

**Key systems:**

- `AccountsDb` — shared `Arc<RwLock<HashMap>>` holding all balances; supports snapshot/rollback for atomic execution
- `PohChain` — running SHA-256 hash chain; `verify_block()` replays hashes to confirm block integrity
- `Mempool` — `Arc<Mutex<VecDeque<Transaction>>>`; API pushes in, the current leader drains each slot
- `LeaderScheduler` — maps `slot → validator_id` using stake-proportional, epoch-seeded deterministic shuffle
- `NetworkBus` — wraps `tokio::broadcast::channel`; simulates P2P gossip between validators
- `ConsensusEngine` — collects votes via `mpsc`; commits state and broadcasts finalization at 3/4 threshold

---

## Project structure

```
solanalite/
├── Cargo.toml
├── src/
│   ├── main.rs              ← starts server + simulation
│   ├── config.rs            ← SimConfig, ValidatorConfig
│   ├── types.rs             ← Transaction, Block, Vote, NetworkMessage, SimEvent
│   ├── error.rs             ← RuntimeError, MempoolError
│   ├── poh.rs               ← PohChain
│   ├── mempool.rs           ← Mempool
│   ├── network.rs           ← NetworkBus
│   ├── scheduler.rs         ← LeaderScheduler
│   ├── consensus.rs         ← ConsensusEngine
│   ├── simulator.rs         ← Orchestrator (wires all tasks + channels)
│   ├── accounts/
│   │   └── db.rs            ← AccountsDb (balances, snapshot/rollback)
│   ├── runtime/
│   │   ├── executor.rs      ← RuntimeExecutor (verify, lock, execute, commit)
│   │   └── builder.rs       ← BlockBuilder
│   ├── validator/
│   │   ├── mod.rs           ← Validator struct + async event loop
│   │   └── state.rs         ← ValidatorState enum
│   └── api/
│       ├── server.rs        ← Axum router + REST handlers
│       └── ws.rs            ← WebSocket handler + event stream
└── frontend/
    ├── src/
    │   ├── hooks/
    │   │   └── useSimulation.ts   ← WebSocket hook, all sim state
    │   └── components/
    │       ├── SlotHeader.tsx
    │       ├── ValidatorCard.tsx
    │       ├── BalanceTable.tsx
    │       ├── BlockFeed.tsx
    │       ├── EventLog.tsx
    │       └── TransactionForm.tsx
    └── vite.config.ts
```

---

## Getting started

### Prerequisites

- Rust stable ≥ 1.75 — [rustup.rs](https://rustup.rs)
- Bun ≥ 1.0 — [bun.sh](https://bun.sh)

### Run the backend

```bash
cargo run
```

The Axum server starts on `http://localhost:3001`.

### Run the frontend

```bash
cd frontend
bun install
bun run dev
```

The dashboard opens at `http://localhost:5173`.

---

## API reference

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/health` | Liveness check |
| `POST` | `/tx` | Submit a transaction to the mempool |
| `GET` | `/balances` | Current account balances |
| `GET` | `/chain` | Full finalized block history |
| `GET` | `/validators` | Validator info (id, name, stake) |
| `GET` | `/ws` | WebSocket — streams all simulation events |

**Submit a transaction:**

```bash
curl -X POST http://localhost:3001/tx \
  -H "Content-Type: application/json" \
  -d '{"from":"Alice","to":"Bob","amount":10}'
```

**WebSocket events** streamed to the frontend:

| Event | When it fires |
|---|---|
| `Snapshot` | Immediately on WebSocket connect |
| `SlotTick` | Every 3 seconds |
| `EpochChange` | Every 10 slots |
| `TransactionQueued` | When a tx enters the mempool |
| `BlockProduced` | When the leader builds a block |
| `VoteReceived` | When a validator votes |
| `BlockFinalized` | When 3/4 approvals reached |
| `BalancesUpdated` | After each finalized block |
| `TransactionResult` | Success or failure per transaction |

---

## Simulation parameters

Configured in `src/config.rs`:

| Parameter | Default | Description |
|---|---|---|
| `epoch_length` | 10 slots | Slots per epoch before schedule recomputes |
| `slot_duration_ms` | 3000 ms | Real-time duration of each slot |
| `genesis_hash` | `"genesis"` | Seed for the initial PoH hash |

**Validator stakes** (initial balances: Alice 100, Bob 20, Charlie 50):

| Validator | Stake | Leader slots per epoch |
|---|---|---|
| V0 | 100 | 2 |
| V1 | 200 | 4 (2× stake) |
| V2 | 100 | 2 |
| V3 | 100 | 2 |

---

## How a transaction flows through the system

1. **Submit** — `POST /tx` pushes the transaction into the `Mempool`
2. **Queue** — `TransactionQueued` event fires; frontend shows pending badge
3. **Slot tick** — slot clock fires; `LeaderScheduler` picks the current leader
4. **Execute** — leader drains the mempool, runs `RuntimeExecutor` (verify → lock → snapshot → transfer → commit/rollback)
5. **PoH fold** — successful transactions are folded into the hash chain
6. **Broadcast** — leader sends the block to all validators via `NetworkBus`
7. **Validate** — each non-leader validator replays the PoH chain and re-executes transactions to verify the state hash
8. **Vote** — validators send `Vote { approved: true/false }` to the `ConsensusEngine`
9. **Finalize** — at 3/4 approvals, the block is committed to `AccountsDb` and appended to the chain
10. **Update** — `BalancesUpdated` event fires; frontend animates the balance change

---

## Consensus threshold

With 4 validators: threshold = `(4 × 2) / 3 + 1` = **3 approvals required**.

If two validators reject (e.g. due to a state hash mismatch), the block receives only 2/4 approvals and never finalizes — it remains pending indefinitely.

---

## License

MIT