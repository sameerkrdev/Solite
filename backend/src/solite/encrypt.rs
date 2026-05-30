use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce, aead::Aead};
use argon2::Argon2;
use rand::{Rng, rngs::OsRng};

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
pub fn aes_encrypt(encryption_key: [u8; 32], data: &str) -> Result<String, ApiError> {
    let key = Key::<Aes256Gcm>::from_slice(&encryption_key);
    let cipher = Aes256Gcm::new(key);

    let nonce_bytes: [u8; 12] = OsRng.r#gen();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let encrypted_bytes = cipher
        .encrypt(nonce, data.as_bytes())
        .map_err(|err| ApiError::Internal(err.to_string()))?;

    let mut combined_bytes = nonce.to_vec();
    combined_bytes.extend_from_slice(&encrypted_bytes);

    let hex_data = hex::encode(combined_bytes);
    Ok(hex_data)
}

// func to decrypt the recovery or private key
pub fn aes_decrypt(encryption_key: [u8; 32], hex_string: &str) -> Result<String, ApiError> {
    let key = Key::<Aes256Gcm>::from_slice(&encryption_key);
    let cipher = Aes256Gcm::new(key);

    let bytes = hex::decode(hex_string).map_err(|e| ApiError::Internal(e.to_string()))?;

    if bytes.len() < 12 {
        return Err(ApiError::Internal("invalid encrypted data".into()));
    }

    let nonce = &bytes[..12];
    let ciphertext = &bytes[12..];

    let nonce = Nonce::from_slice(&nonce);

    let decrypted_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let decrypted_string =
        String::from_utf8(decrypted_bytes.into()).map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(decrypted_string)
}
