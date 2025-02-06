use shuttle_runtime::Error as ShuttleError;
use teloxide::{ApiError, RequestError};

use crate::platform::PlatformError;
use crate::runtime::RuntimeError;
use crate::{config::ConfigError, service::ServiceError, storage::StorageError};

#[derive(Debug, thiserror::Error)]
pub enum BotError {
    #[error("Service error: {0}")] // check
    ServiceError(#[from] ServiceError),

    #[error("Dialogue state error: {0}")] // check
    DialogueStateError(String),

    #[error("App state error: {0}")] // check
    AppStateError(String),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Config error: {0}")] // check
    ConfigError(#[from] ConfigError),

    #[error("Platform error: {0}")]
    PlatformError(#[from] PlatformError),

    #[error("Runtime error: {0}")]
    RuntimeError(#[from] RuntimeError),

    #[error(transparent)]
    Other(anyhow::Error), // check
}

impl From<BotError> for ShuttleError {
    fn from(error: BotError) -> Self {
        ShuttleError::Custom(anyhow::anyhow!(error))
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
