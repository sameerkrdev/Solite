## Backend Core
- [x] Init backend
- [x] Basic Axum server
- [x] Health route
- [x] AccountsDb with ensure_account (ed25519 validation)
- [x] Airdrop endpoint
- [ ] CORS
- [ ] Error handling + global config
- [ ] Transaction endpoint (POST /tx with signature verify)
- [ ] GET /chain
- [ ] GET /wallet/:address
- [ ] WebSocket event stream

## Simulation Engine
- [ ] Mempool
- [ ] PoH chain
- [ ] Leader scheduler
- [ ] Slot clock
- [ ] Validator tasks (4 validators)
- [ ] Consensus engine
- [ ] Simulation orchestrator (wire everything)

## Frontend
- [ ] Create wallet (generate ed25519, store in memory)
- [ ] Add existing wallet (import pubkey + privkey)
- [ ] Home page — view all wallets, switch wallet
- [ ] View balance + transaction history per wallet
- [ ] Transaction submit form (sign + POST /tx)
- [ ] Slot header (slot, epoch, leader)
- [ ] Validator cards x4
- [ ] Balance table (live updates)
- [ ] Block feed (finalized blocks)
- [ ] Event log (WebSocket stream)