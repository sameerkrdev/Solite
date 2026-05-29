use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("missing field: {0}")]
    MissingField(String),

    #[error("invalid address format: {0}")]
    InvalidAddress(String),

    #[error("invalid amount: {0}")]
    InvalidAmount(String),

    #[error("wallet not found: {0}")]
    WalletNotFound(String),

    #[error("insufficient balance: {account} has {have}, needs {need}")]
    InsufficientBalance {
        account: String,
        have: u64,
        need: u64,
    },

    #[error("cannot send to yourself")]
    SelfTransfer,

    #[error("missing signature")]
    MissingSignature,

    #[error("airdrop amount exceeds limit of {limit}")]
    AirdropLimitExceeded { limit: u64 },

    #[error("mempool is full, try again later")]
    MempoolFull,

    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            ApiError::MissingField(_) => (StatusCode::BAD_REQUEST, "MISSING_FIELD"),
            ApiError::InvalidAddress(_) => (StatusCode::BAD_REQUEST, "INVALID_ADDRESS"),
            ApiError::InvalidAmount(_) => (StatusCode::BAD_REQUEST, "INVALID_AMOUNT"),
            ApiError::WalletNotFound(_) => (StatusCode::BAD_REQUEST, "WALLET_NOT_FOUND"),
            ApiError::InsufficientBalance { .. } => {
                (StatusCode::BAD_REQUEST, "INSUFFICIENT_BALANCE")
            }
            ApiError::SelfTransfer => (StatusCode::BAD_REQUEST, "SELF_TRANSFER"),
            ApiError::MissingSignature => (StatusCode::BAD_REQUEST, "MISSING_SIGNATURE"),
            ApiError::AirdropLimitExceeded { .. } => {
                (StatusCode::UNPROCESSABLE_ENTITY, "AIRDROP_LIMIT")
            }
            ApiError::MempoolFull => (StatusCode::SERVICE_UNAVAILABLE, "MEMPOOL_FULL"),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };

        let body = json!({
            "ok":    false,
            "code":  code,           // machine readable — frontend switches on this
            "error": self.to_string() // human readable — from thiserror #[error(...)]
        });

        (status, Json(body)).into_response()
    }
}
