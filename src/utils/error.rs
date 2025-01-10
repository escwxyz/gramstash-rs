// TODO: Add more comprehensive error handling

use redis::RedisError;
use shuttle_runtime::Error as ShuttleError;
use teloxide::{ApiError, RequestError};
use thiserror::Error;

// #[derive(Debug, Error)]
// pub enum DialogueStateError {
//     #[error("Invalid state transition: {from} -> {to}")]
//     InvalidStateTransition { from: String, to: String },
//     #[error("State not found: {0}")]
//     StateNotFound(String),
//     #[error("Session expired")]
//     SessionExpired,
//     #[error("Storage error: {0}")]
//     StorageError(String),
// }

// #[derive(Debug, Error)]
// pub enum TelegramError {
//     #[error("API error: {0}")]
//     ApiError(String),
//     #[error("Network error: {0}")]
//     NetworkError(#[from] reqwest::Error),
//     // #[error("Rate limit exceeded. Try again in {0:?}")]
//     // RateLimited(Duration),
//     #[error("Invalid bot token")]
//     InvalidToken,
//     #[error("Message not found: {0}")]
//     MessageNotFound(String),
// }

// #[derive(Debug, thiserror::Error)]
// pub enum AuthenticationError {
//     #[error("Login failed: Bad credentials")]
//     BadCredentials,
//     #[error("Two-factor authentication required")]
//     TwoFactorRequired,
//     #[error("Checkpoint verification required: {0}")]
//     CheckpointRequired(String),
//     #[error("Login failed: {0}")]
//     LoginFailed(String),
//     #[error("Network error: {0}")]
//     NetworkError(#[from] reqwest::Error),
// }

// #[derive(Debug, thiserror::Error)]
// pub enum InstagramAPIError {
//     #[error("Authentication required")]
//     AuthRequired,
//     #[error("API error: {0}")]
//     APIError(String),
//     #[error("Invalid URL: {0}")]
//     InvalidUrl(String),
//     #[error("Media not found: {0}")]
//     MediaNotFound(String),
//     #[error("Network error: {0}")]
//     NetworkError(#[from] reqwest::Error),
// }

#[derive(Debug, Error)]
pub enum BotError {
    #[error("Instagram API error: {0}")]
    InstagramApi(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Media not found: {0}")]
    MediaNotFound(String),

    #[error("Authentication required: {0}")]
    AuthRequired(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Dialogue error: {0}")]
    DialogueError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Redis error: {0}")]
    RedisError(String),

    #[error("App state error: {0}")]
    AppStateError(String),

    #[error(transparent)]
    Other(anyhow::Error),
    // TODO: implement more comprehensive errors
}

impl From<BotError> for ShuttleError {
    fn from(error: BotError) -> Self {
        ShuttleError::Custom(anyhow::anyhow!(error))
    }
}
impl From<RedisError> for BotError {
    fn from(error: RedisError) -> Self {
        BotError::RedisError(error.to_string())
    }
}

impl From<BotError> for RequestError {
    fn from(error: BotError) -> Self {
        RequestError::Api(ApiError::Unknown(error.to_string()))
    }
}

impl From<anyhow::Error> for BotError {
    fn from(error: anyhow::Error) -> Self {
        BotError::Other(error)
    }
}

pub type HandlerResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub type BotResult<T> = Result<T, BotError>;
