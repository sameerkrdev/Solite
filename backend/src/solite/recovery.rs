use crate::error_handler::ApiError;
use bip39::{Language, Mnemonic};

// func to generate a new recovery phrase
pub fn generate_recovery_phrase() -> Result<String, ApiError> {
    let mnemonic = match Mnemonic::generate_in(Language::English, 24) {
        Ok(value) => value,
        Err(e) => return Err(ApiError::Internal(e.to_string())),
    };

    let recovery_phrase = mnemonic.words().collect::<Vec<_>>().join(" ");

    Ok(recovery_phrase)
}

// func to validate the phrase
pub fn validate_phrase(phrase: &str) -> Result<Mnemonic, ApiError> {
    match Mnemonic::parse(phrase) {
        Ok(value) => Ok(value),
        Err(e) => Err(ApiError::InvalidRecoveryPhrase(e.to_string())),
    }
}

// func to generate the seed from recovery phrase
pub fn generate_seed(phrase: &str) -> Result<[u8; 64], ApiError> {
    let mnemonic = validate_phrase(phrase)?;

    let seed = mnemonic.to_seed("");

    Ok(seed)
}
