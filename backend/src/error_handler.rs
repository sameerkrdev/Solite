use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("missing field: {0}")]
    MissingField(String),

    #[error("invalid address format: {0}")]
    InvalidAddress(String),

    #[error("invalid recovery phrase: {0}")]
    InvalidRecoveryPhrase(String),

    #[error("invalid private key: {0}")]
    InvalidPrivateKey(String),

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

    #[error("invalid credentials")]
    Unauthorized,

    #[error("invalid wallet password")]
    InvalidWalletPassword,

    #[error("forbidden")]
    Forbidden,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if matches!(self, ApiError::Internal(ref msg) if !msg.is_empty()) {
            error!(error = %self, "internal api error");
        }

        let (status, code, message) = match &self {
            ApiError::MissingField(_) => (StatusCode::BAD_REQUEST, "MISSING_FIELD", self.to_string()),
            ApiError::InvalidAddress(_) => {
                (StatusCode::BAD_REQUEST, "INVALID_ADDRESS", self.to_string())
            }
            ApiError::InvalidRecoveryPhrase(_) => {
                (StatusCode::BAD_REQUEST, "INVALID_PHRASE", self.to_string())
            }
            ApiError::InvalidPrivateKey(_) => {
                (StatusCode::BAD_REQUEST, "INVALID_PRIVATE_KEY", self.to_string())
            }
            ApiError::InvalidAmount(_) => (StatusCode::BAD_REQUEST, "INVALID_AMOUNT", self.to_string()),
            ApiError::WalletNotFound(_) => {
                (StatusCode::NOT_FOUND, "WALLET_NOT_FOUND", self.to_string())
            }
            ApiError::InsufficientBalance { .. } => (
                StatusCode::BAD_REQUEST,
                "INSUFFICIENT_BALANCE",
                self.to_string(),
            ),
            ApiError::SelfTransfer => (StatusCode::BAD_REQUEST, "SELF_TRANSFER", self.to_string()),
            ApiError::MissingSignature => {
                (StatusCode::BAD_REQUEST, "MISSING_SIGNATURE", self.to_string())
            }
            ApiError::AirdropLimitExceeded { .. } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "AIRDROP_LIMIT",
                self.to_string(),
            ),
            ApiError::MempoolFull => (
                StatusCode::SERVICE_UNAVAILABLE,
                "MEMPOOL_FULL",
                self.to_string(),
            ),
            ApiError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "internal error".to_string(),
            ),
            ApiError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND", self.to_string()),
            ApiError::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT", self.to_string()),
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "invalid credentials".to_string(),
            ),
            ApiError::InvalidWalletPassword => (
                StatusCode::UNAUTHORIZED,
                "INVALID_WALLET_PASSWORD",
                "invalid wallet password".to_string(),
            ),
            ApiError::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "not allowed".to_string(),
            ),
        };

        let body = json!({
            "ok": false,
            "code": code,
            "error": message
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

    #[error("wallet not found: {0}")]
    WalletNotFound(String),
}

impl From<SoliteError> for ApiError {
    fn from(e: SoliteError) -> Self {
        match e {
            SoliteError::UserNotFound(_) => ApiError::NotFound(e.to_string()),
            SoliteError::WalletNotFound(addr) => ApiError::WalletNotFound(addr),
            SoliteError::UsernameTaken(_)
            | SoliteError::EmailTaken(_)
            | SoliteError::AddressAlreadyExists(_)
            | SoliteError::GoogleAlreadyLinked => ApiError::Conflict(e.to_string()),
        }
    }
}
