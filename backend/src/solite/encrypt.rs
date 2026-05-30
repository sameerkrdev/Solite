use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce, aead::Aead};
use argon2::Argon2;
// use tower_http::classify::GrpcCode::Ok;

use crate::error_handler::ApiError;

// func to generate encryption key --> via password or google auth use pin
pub fn generate_aes_encryption_key(secret: &str, salt: &str) -> Result<[u8; 32], ApiError> {
    // let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();
    let mut key = [0u8; 32]; // AES-256 = 32 bytes

    argon2
        .hash_password_into(secret.as_bytes(), salt.as_bytes(), &mut key)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(key)
}

// func to encrypt the recovery or private key
pub fn aes_encrypt(
    encryption_key: [u8; 32],
    nonce: [u8; 12],
    data: &str,
) -> Result<String, ApiError> {
    let key = Key::<Aes256Gcm>::from_slice(&encryption_key);

    let cipher = Aes256Gcm::new(key);

    let nonce = Nonce::from_slice(&nonce);

    let encrypted_data = cipher
        .encrypt(nonce, data.as_bytes())
        .map_err(|err| ApiError::Internal(err.to_string()))?;

    Ok(hex::encode(encrypted_data))
}

// func to decrypt the recovery or private key
pub fn aes_decrypt(
    encryption_key: [u8; 32],
    nonce: [u8; 12],
    encrypted_data: &str,
) -> Result<String, ApiError> {
    let key = Key::<Aes256Gcm>::from_slice(&encryption_key);

    let cipher = Aes256Gcm::new(key);

    let nonce = Nonce::from_slice(&nonce);

    let decrypted_data = cipher
        .decrypt(nonce, encrypted_data.as_bytes())
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let data_str = hex::encode(decrypted_data);

    Ok(data_str)
}
