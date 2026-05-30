use rand::{Rng, rngs::OsRng};

use crate::{
    error_handler::ApiError,
    solite::{
        db::wallet::{WalletEntry, WalletSecret},
        encrypt::{aes_encrypt, derive_key_from_password},
        keypair::{self, Keypair},
        recovery::{generate_recovery_phrase, validate_phrase},
    },
};

fn generate_kdf_salt() -> String {
    hex::encode(OsRng.r#gen::<[u8; 16]>())
}

fn encode_address(pubkey: [u8; 32]) -> String {
    bs58::encode(pubkey).into_string()
}

fn build_wallet_with_phrase(
    password: &str,
    phrase: &str,
    label: &str,
) -> Result<WalletEntry, ApiError> {
    let keypair = keypair::generate_keypair(phrase)?;
    let address = encode_address(keypair.public_key);
    let kdf_salt = generate_kdf_salt();
    let aes_key = derive_key_from_password(password, &kdf_salt)?;
    let encrypted_recovery_phrase = aes_encrypt(aes_key, phrase)?;

    Ok(WalletEntry::new(
        label.to_string(),
        address,
        WalletSecret::WithPhrase {
            encrypted_recovery_phrase,
        },
        kdf_salt,
    ))
}

pub fn create_generated_wallet(
    password: &str,
    label: &str,
) -> Result<(WalletEntry, String), ApiError> {
    let phrase = generate_recovery_phrase()?;
    let entry = build_wallet_with_phrase(password, &phrase, label)?;
    Ok((entry, phrase))
}

pub fn import_phrase_wallet(
    password: &str,
    phrase: &str,
    label: &str,
) -> Result<WalletEntry, ApiError> {
    validate_phrase(phrase)?;
    build_wallet_with_phrase(password, phrase, label)
}

pub fn import_privkey_wallet(
    password: &str,
    privkey_b58: &str,
    label: &str,
) -> Result<WalletEntry, ApiError> {
    if !keypair::is_valid_ed25519_privkey(privkey_b58) {
        return Err(ApiError::InvalidPrivateKey(
            "invalid ed25519 private key".into(),
        ));
    }

    let privkey_bytes: Vec<u8> = bs58::decode(privkey_b58)
        .into_vec()
        .map_err(|_| ApiError::InvalidPrivateKey("invalid base58 private key".into()))?;

    let privkey_arr: [u8; 32] = privkey_bytes
        .try_into()
        .map_err(|_| ApiError::InvalidPrivateKey("private key must be 32 bytes".into()))?;

    let pubkey = keypair::pubkey_from_privkey(privkey_arr);
    let address = encode_address(pubkey);
    let kdf_salt = generate_kdf_salt();
    let aes_key = derive_key_from_password(password, &kdf_salt)?;
    let privkey_hex = hex::encode(privkey_arr);
    let encrypted_private_key = aes_encrypt(aes_key, &privkey_hex)?;

    Ok(WalletEntry::new(
        label.to_string(),
        address,
        WalletSecret::WithPrivateKey {
            encrypted_private_key,
        },
        kdf_salt,
    ))
}

pub fn unlock_keypair(entry: &WalletEntry, password: &str) -> Result<Keypair, ApiError> {
    let aes_key = derive_key_from_password(password, &entry.kdf_salt)?;

    match &entry.secret {
        WalletSecret::WithPhrase {
            encrypted_recovery_phrase,
        } => {
            let phrase = crate::solite::encrypt::aes_decrypt(aes_key, encrypted_recovery_phrase)?;
            keypair::generate_keypair(&phrase)
        }
        WalletSecret::WithPrivateKey {
            encrypted_private_key,
        } => {
            let privkey_hex =
                crate::solite::encrypt::aes_decrypt(aes_key, encrypted_private_key)?;
            let privkey_bytes: [u8; 32] = hex::decode(&privkey_hex)
                .map_err(|_| ApiError::InvalidWalletPassword)?
                .try_into()
                .map_err(|_| ApiError::InvalidWalletPassword)?;

            let public_key = keypair::pubkey_from_privkey(privkey_bytes);
            Ok(Keypair {
                private_key: privkey_bytes,
                public_key,
            })
        }
    }
}

pub fn decrypt_recovery_phrase(entry: &WalletEntry, password: &str) -> Result<String, ApiError> {
    match &entry.secret {
        WalletSecret::WithPhrase {
            encrypted_recovery_phrase,
        } => {
            let aes_key = derive_key_from_password(password, &entry.kdf_salt)?;
            crate::solite::encrypt::aes_decrypt(aes_key, encrypted_recovery_phrase)
        }
        WalletSecret::WithPrivateKey { .. } => Err(ApiError::NotFound(
            "recovery phrase not available for private key wallets".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_wallet_unlock_roundtrip() {
        let (entry, phrase) = create_generated_wallet("wallet-pwd", "main").unwrap();
        let kp = unlock_keypair(&entry, "wallet-pwd").unwrap();
        let kp2 = keypair::generate_keypair(&phrase).unwrap();
        assert_eq!(kp.public_key, kp2.public_key);
        assert_eq!(kp.private_key, kp2.private_key);
    }

    #[test]
    fn privkey_import_unlock_roundtrip() {
        let seed = create_generated_wallet("pwd", "seed").unwrap();
        let kp = unlock_keypair(&seed.0, "pwd").unwrap();
        let privkey_b58 = bs58::encode(kp.private_key).into_string();

        let entry = import_privkey_wallet("pwd", &privkey_b58, "imported").unwrap();
        let kp2 = unlock_keypair(&entry, "pwd").unwrap();
        assert_eq!(kp.public_key, kp2.public_key);
    }
}
