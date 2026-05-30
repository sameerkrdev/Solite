use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce, aead::Aead};
use argon2::Argon2;
use rand::{Rng, rngs::OsRng};

use crate::error_handler::ApiError;

pub fn derive_key_from_password(password: &str, salt_hex: &str) -> Result<[u8; 32], ApiError> {
    let salt = hex::decode(salt_hex).map_err(|e| ApiError::Internal(e.to_string()))?;

    let argon2 = Argon2::default();
    let mut key = [0u8; 32];

    argon2
        .hash_password_into(password.as_bytes(), &salt, &mut key)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(key)
}

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

    Ok(hex::encode(combined_bytes))
}

pub fn aes_decrypt(encryption_key: [u8; 32], hex_string: &str) -> Result<String, ApiError> {
    let key = Key::<Aes256Gcm>::from_slice(&encryption_key);
    let cipher = Aes256Gcm::new(key);

    let bytes = hex::decode(hex_string).map_err(|_| ApiError::InvalidWalletPassword)?;

    if bytes.len() < 12 {
        return Err(ApiError::InvalidWalletPassword);
    }

    let (nonce_bytes, ciphertext) = bytes.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let decrypted_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| ApiError::InvalidWalletPassword)?;

    String::from_utf8(decrypted_bytes).map_err(|_| ApiError::InvalidWalletPassword)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes_roundtrip() {
        let key = derive_key_from_password("test-password", &hex::encode([1u8; 16])).unwrap();
        let encrypted = aes_encrypt(key, "hello world").unwrap();
        let decrypted = aes_decrypt(key, &encrypted).unwrap();
        assert_eq!(decrypted, "hello world");
    }

    #[test]
    fn nonce_is_prepended() {
        let key = derive_key_from_password("pwd", &hex::encode([2u8; 16])).unwrap();
        let encrypted = aes_encrypt(key, "data").unwrap();
        let bytes = hex::decode(&encrypted).unwrap();
        assert!(bytes.len() > 12);
    }

    #[test]
    fn derive_key_uses_hex_salt_not_utf8() {
        let salt_hex = hex::encode([0xABu8; 16]);
        let k1 = derive_key_from_password("pwd", &salt_hex).unwrap();
        let k2 = derive_key_from_password("pwd", &salt_hex).unwrap();
        assert_eq!(k1, k2);
    }
}
