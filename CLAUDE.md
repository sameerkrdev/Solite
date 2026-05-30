# Solite — Custom Phantom Wallet Auth

Backend wallet auth system for Solite. Password-protected wallets with encrypted recovery phrases or private keys, JWT session auth, and Google identity (Option 1: ID token verification).

## Quick start

```bash
export JWT_SECRET=your-secret-min-32-chars
export GOOGLE_CLIENT_ID=your-google-oauth-client-id

cd backend
cargo run
# Server: http://127.0.0.1:3000
```

**Postman collection:** [`backend/postman/Custom-Phantom-Wallet-Auth.postman_collection.json`](backend/postman/Custom-Phantom-Wallet-Auth.postman_collection.json)

Import into Postman, set collection variables (`username`, `password`, `wallet_password`), run **Flow 1** first to populate tokens and `recovery_phrase`.

---

## What is complete

| Area | Status | Location |
|------|--------|----------|
| Crypto (Argon2id + AES-256-GCM) | Done | `backend/src/solite/encrypt.rs` |
| BIP39 recovery + SLIP-10 key derivation | Done | `backend/src/solite/recovery.rs`, `keypair.rs` |
| Wallet orchestration (6 flows + unlock) | Done | `backend/src/solite/wallet_service.rs` |
| In-memory user DB | Done | `backend/src/solite/db/` |
| Password hash / verify | Done | `backend/src/solite/auth/password.rs` |
| JWT access + refresh tokens | Done | `backend/src/solite/auth/jwt.rs` |
| Google ID token verify (JWKS) | Done | `backend/src/solite/auth/google.rs` |
| JWT middleware | Done | `backend/src/api/middleware/auth.rs` |
| All HTTP handlers | Done | `backend/src/api/handlers/solite/` |
| Unit tests (encrypt + wallet unlock) | Done | `encrypt.rs`, `wallet_service.rs` |
| Simulation / chain / mempool | Not started | — |
| Frontend onboarding UI | Not started | `frontend/` |

---

## Cryptographic model

- **Login auth:** Argon2id verification → `User.password` (stored hash)
- **Wallet crypto:** Argon2id KDF → 32-byte AES-256-GCM key via `derive_key_from_password(password, kdf_salt_hex)`
- AES key is **never stored** — derived on every unlock/sign/export
- `kdf_salt`: 16 random bytes, hex-encoded, plaintext per wallet in `WalletEntry`
- **Phrase wallets:** encrypted BIP39 phrase → `WalletSecret::WithPhrase`; private key derived on demand
- **Privkey wallets:** encrypted hex privkey → `WalletSecret::WithPrivateKey`; no recovery phrase
- AES nonce: fresh per encrypt, prepended to ciphertext, stored as single hex string

---

## Six wallet flows

### New user (at register) — Flows 1–3

All use `POST /api/v1/auth/register`. Password is used for both login hash and wallet KDF.

| Flow | `wallet.mode` | Extra fields | Stored secret | DB |
|------|---------------|--------------|---------------|-----|
| **1 — Generate** | `generate` | `label` | `WithPhrase` | `userdb.insert` |
| **2 — Import phrase** | `import_phrase` | `label`, `phrase` | `WithPhrase` | `insert` + `owner_of` check |
| **3 — Import privkey** | `import_privkey` | `label`, `private_key` (base58) | `WithPrivateKey` | `insert` + `owner_of` check |

**Flow 1 response** includes `recovery_phrase` once — only time server exposes it for generated wallets.

**Register body example (Flow 1):**
```json
{
  "username": "alice",
  "password": "secure-pass",
  "email": "alice@example.com",
  "wallet": { "mode": "generate", "label": "main" }
}
```

**How it works internally:**
1. `generate_recovery_phrase()` or `validate_phrase()` / `is_valid_ed25519_privkey()`
2. Derive address from keypair (`bs58::encode(pubkey)`) — never trust client address
3. Fresh `kdf_salt` → `derive_key_from_password` → `aes_encrypt`
4. `hash_password(password)` → `User::new` → `userdb.insert`
5. Issue JWT access + refresh tokens

---

### Existing user (JWT protected) — Flows 4–6

Login first (`POST /api/v1/auth/login`), then use `Authorization: Bearer <access_token>`.

| Flow | Endpoint | Body | Stored secret |
|------|----------|------|---------------|
| **4 — Generate** | `POST /api/v1/user/wallet/new` | `wallet_password`, `label` | `WithPhrase` |
| **5 — Import phrase** | `POST /api/v1/user/wallet/import` | `mode: "phrase"`, `wallet_password`, `label`, `phrase` | `WithPhrase` |
| **6 — Import privkey** | `POST /api/v1/user/wallet/import` | `mode: "privkey"`, `wallet_password`, `label`, `private_key` | `WithPrivateKey` |

For regular users, `wallet_password` = login `password`. Google users use `wallet_password` set at Google register.

**Flow 4 response** includes `recovery_phrase` once.

**How it works internally:**
1. JWT validates `user_id`
2. Same crypto pipeline as register flows
3. `userdb.add_wallet(user_id, entry)` (imports check `owner_of`)

---

## Signing & export

| Action | Endpoint | Auth |
|--------|----------|------|
| Export recovery phrase | `GET /api/v1/user/recovery-phrase?address=&wallet_password=` | JWT + password |
| Export private key | `GET /api/v1/user/private-key?address=&wallet_password=` | JWT + password |
| Sign message | `POST /api/v1/user/sign` | JWT + password |

**Sign body:** `{ "address", "wallet_password", "message" }` where `message` is hex-encoded bytes.

**Unlock path (`wallet_service::unlock_keypair`):**
- `WithPhrase` → decrypt phrase → `generate_keypair(phrase)` → sign
- `WithPrivateKey` → decrypt hex privkey → sign directly

---

## Auth endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/auth/register` | Flows 1–3 |
| POST | `/api/v1/auth/login` | username + password → JWT |
| POST | `/api/v1/auth/refresh` | refresh token → new tokens |
| POST | `/api/v1/auth/google/register` | Google ID token + wallet_password + optional wallet |
| POST | `/api/v1/auth/google/login` | Google ID token → JWT |
| GET | `/api/v1/health/health` | Health check |

## User endpoints (JWT required)

| Method | Path |
|--------|------|
| GET | `/api/v1/user/me` |
| GET | `/api/v1/user/wallets` |
| POST | `/api/v1/user/wallet/new` |
| POST | `/api/v1/user/wallet/import` |
| GET | `/api/v1/user/recovery-phrase` |
| GET | `/api/v1/user/private-key` |
| POST | `/api/v1/user/sign` |

---

## Google auth

- **Option 1:** Frontend obtains Google ID token (GIS); backend verifies via Google JWKS
- `google_id` is identity only — never used for wallet KDF
- Wallet crypto always uses `wallet_password` (Argon2 hash stored in `User.password`)

---

## Environment variables

| Variable | Required | Default |
|----------|----------|---------|
| `JWT_SECRET` | Yes | — |
| `GOOGLE_CLIENT_ID` | Yes | — |
| `JWT_ACCESS_EXPIRY_SECS` | No | 900 |
| `JWT_REFRESH_EXPIRY_SECS` | No | 604800 |

---

## Module map

```
backend/src/solite/
  recovery.rs       — BIP39 phrase generate/validate/seed
  keypair.rs        — SLIP-10 m/44'/501'/0'/0' derivation, privkey validation
  encrypt.rs        — derive_key_from_password, aes_encrypt, aes_decrypt
  wallet_service.rs — create/import/unlock orchestration
  auth/
    password.rs     — Argon2id hash/verify
    jwt.rs          — Solite JWT issue/verify
    google.rs       — Google ID token verify
  db/
    user.rs         — User struct
    wallet.rs       — WalletEntry, WalletSecret
    store.rs        — UserDb (in-memory)
```

---

## Suggested Postman test order

1. **Flow 1** — Register + generate → saves tokens, `recovery_phrase`, `address`
2. **Login** — re-auth if needed
3. **Get Me** / **List Wallets** — confirm wallet metadata
4. **Export Recovery Phrase** — verify decrypt works
5. **Export Private Key** — saves `private_key` variable
6. **Sign Message** — verify signature returned
7. **Flow 4** — add second generated wallet
8. **Flow 5** — import phrase (use saved `recovery_phrase`)
9. **Flow 6** — import privkey (use saved `private_key`)

Flows 2–3 at register use different usernames to avoid conflicts.

---

## Security rules (enforced)

- Raw password never stored; AES key never stored
- Address always derived from keys — never from client input
- Internal errors logged server-side; generic messages to client
- DB breach exposes only argon2 hashes, kdf_salts, encrypted blobs — useless without password
