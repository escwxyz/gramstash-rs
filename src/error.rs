use redis::RedisError;
use shuttle_runtime::Error as ShuttleError;
use teloxide::{ApiError, RequestError};

#[derive(Debug, thiserror::Error)]
pub enum MiddlewareError {
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Other error: {0}")]
    Other(String),
}

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Login failed: Bad credentials")]
    BadCredentials,
    #[error("Two-factor authentication required")]
    TwoFactorRequired,
    #[error("Checkpoint verification required: {0}")]
    CheckpointRequired(String),
    #[error("Login failed: {0}")]
    LoginFailed(String),
    #[error("Other error: {0}")]
    Other(String),
}

impl From<reqwest::Error> for AuthError {
    fn from(error: reqwest::Error) -> Self {
        AuthError::Other(error.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthenticationError {
    #[error("Login failed: Bad credentials")]
    BadCredentials,
    #[error("Two-factor authentication required")]
    TwoFactorRequired,
    #[error("Checkpoint verification required: {0}")]
    CheckpointRequired(String),
    #[error("Login failed: {0}")]
    LoginFailed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum InstagramError {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error("Invalid response structure: {0}")]
    InvalidResponseStructure(String),
    #[error("Auth error: {0}")]
    AuthenticationError(#[from] AuthenticationError),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Content not found: {0}")]
    ContentNotFound(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Invalid session")]
    InvalidSession,
    #[error("Session not found")]
    SessionNotFound,
    #[error("Session expired")]
    SessionExpired,
    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Cache: {0}")] // check
    Cache(String),
    #[error("Instagram: {0}")] // check
    InstagramError(#[from] InstagramError),
    #[error("Session: {0}")]
    Session(#[from] SessionError),
    #[error("Middleware: {0}")] // check
    Middleware(#[from] MiddlewareError),
    #[error("Language: {0}")]
    Language(String),
}

#[derive(Debug, thiserror::Error)]
pub enum BotError {
    #[error("Error loading secret key: {0}")] // check
    SecretKeyError(String),

    #[error("Service error: {0}")] // check
    ServiceError(ServiceError),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String), // TODO parse error...

    #[error("Dialogue state error: {0}")] // check
    DialogueStateError(String),

    #[error("Redis error: {0}")] // check
    RedisError(String),

    #[error("Turso error: {0}")] // check
    TursoError(String),

    #[error("App state error: {0}")] // check
    AppStateError(String),

    #[error(transparent)]
    Other(anyhow::Error), // check
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
// TODO: check
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
