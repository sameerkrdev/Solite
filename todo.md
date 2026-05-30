## Backend Core
- [x] Init backend
- [x] Basic Axum server
- [x] Health route
- [x] AccountsDb with ensure_account (ed25519 validation)
- [x] Airdrop endpoint
- [x] CORS
- [x] Error handling + global config
- [ ] User system
    - [x] Data Model setup
    - [x] Basic Route to create a user
    - [ ] 
    - [ ] Jwt token --> (Optional) Refresh/Access Token + Private/Public Key
    - [ ] Get user info with all wallets id
    - [ ] Get recovery phrase
    - [ ] Get private key
    - [ ] Sign the transcation with private key --> memepool
    - [ ] Note: 
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


- New User with no wallet --> username/password or google --> Create the recovery phrase --> Create the Public/Private key --> store all
- New User having a wallet --> username/password or google --> google/private key/recovery phrase
- Existing user want to add create a new wallet --> Create the recovery phrase --> Create the Public/Private key --> store all
- Existing user want to add new wallet --> google/private key/recovery phrase