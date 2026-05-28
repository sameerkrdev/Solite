use ed25519_dalek::VerifyingKey;

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
