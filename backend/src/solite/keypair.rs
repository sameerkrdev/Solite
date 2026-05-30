use ed25519_dalek::{SigningKey, VerifyingKey};
use slip10::{BIP32Path, Curve, derive_key_from_path};
use std::str::FromStr;

use crate::{error_handler::ApiError, solite::recovery::generate_seed};

// func to generate a new keypair via recovery phrase --> take: raw recovery phrase | give: private and public key
pub struct Keypair {
    pub private_key: [u8; 32],
    pub public_key: [u8; 32],
}
pub fn generate_keypair(phrase: &str) -> Result<Keypair, ApiError> {
    let seed = generate_seed(phrase)?;

    let path = match BIP32Path::from_str("m/44'/501'/0'/0'") {
        Ok(value) => value,
        Err(e) => return Err(ApiError::Internal(e.to_string())),
    };

    let derived = match derive_key_from_path(seed.as_slice(), Curve::Ed25519, &path) {
        Ok(value) => value,
        Err(e) => return Err(ApiError::Internal(e.to_string())),
    };

    // First 32 bytes are used as Ed25519 secret key material
    let signing_key = SigningKey::from_bytes(
        &derived.key[..32]
            .try_into()
            .map_err(|_| ApiError::Internal("failed to derive 32-byte key".into()))?,
    );

    let verifying_key = signing_key.verifying_key();

    let keypair = Keypair {
        private_key: signing_key.as_bytes().clone(),
        public_key: verifying_key.as_bytes().clone(),
    };

    Ok(keypair)
}

pub fn pubkey_from_privkey(private_key: [u8; 32]) -> [u8; 32] {
    let signing_key = SigningKey::from_bytes(&private_key);
    signing_key.verifying_key().as_bytes().clone()
}

// func to validate the public
pub fn is_valid_ed25519_pubkey(key: &str) -> bool {
    let bytes = match bs58::decode(key).into_vec() {
        Ok(b) => b,
        Err(_) => return false,
    };

    if bytes.len() != 32 {
        return false;
    }

    let arr: [u8; 32] = bytes.try_into().unwrap();

    VerifyingKey::from_bytes(&arr).is_ok()
}

// SigningKey::from_bytes always succeeds. Any 32 bytes is a valid ed25519 private key mathematically
pub fn is_valid_ed25519_privkey(key: &str) -> bool {
    let bytes = match bs58::decode(key).into_vec() {
        Ok(b) => b,
        Err(_) => return false,
    };

    if bytes.len() != 32 {
        return false;
    }

    true
}
