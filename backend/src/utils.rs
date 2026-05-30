use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{
    Argon2, PasswordHasher, PasswordVerifier,
    password_hash::{self, PasswordHash, SaltString, rand_core::OsRng},
};
use ed25519_dalek::VerifyingKey;

use crate::error_handler::ApiError;

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

pub fn unix_ms_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

pub fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .to_string();

    Ok(password_hash)
}

pub fn compare_hash_password(hash: &str, password: &str) -> Result<bool, ApiError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| ApiError::Internal(e.to_string()))?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(password_hash::Error::Password) => Ok(false),
        Err(e) => Err(ApiError::Internal(e.to_string())),
    }
}
