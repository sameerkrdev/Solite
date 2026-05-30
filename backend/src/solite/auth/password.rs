use argon2::{
    Argon2, PasswordHasher, PasswordVerifier,
    password_hash::{self, PasswordHash, SaltString, rand_core::OsRng},
};

use crate::error_handler::ApiError;

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
