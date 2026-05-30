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

    #[error("not found: {0}")]
    NotFound(String),

    #[error("data conflict: {0}")]
    Conflict(String),
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
            ApiError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            ApiError::Conflict(_) => (StatusCode::BAD_REQUEST, "CONFLICT"),
        };

        let body = json!({
            "ok":    false,
            "code":  code,           // machine readable — frontend switches on this
            "error": self.to_string() // human readable — from thiserror #[error(...)]
        });

        (status, Json(body)).into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SoliteError {
    #[error("user not found: {0}")]
    UserNotFound(String),

    #[error("username already taken: {0}")]
    UsernameTaken(String),

    #[error("email already taken: {0}")]
    EmailTaken(String),

    #[error("wallet address already exists: {0}")]
    AddressAlreadyExists(String),

    #[error("google account already linked")]
    GoogleAlreadyLinked,
}

impl From<SoliteError> for ApiError {
    fn from(e: SoliteError) -> Self {
        match e {
            SoliteError::UserNotFound(_) => ApiError::NotFound(e.to_string()),
            SoliteError::UsernameTaken(_)
            | SoliteError::EmailTaken(_)
            | SoliteError::AddressAlreadyExists(_)
            | SoliteError::GoogleAlreadyLinked => ApiError::Conflict(e.to_string()),
        }
    }
}
