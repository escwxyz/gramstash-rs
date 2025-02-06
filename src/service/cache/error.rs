use crate::{error::BotError, service::ServiceError, storage::StorageError};

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Cache error: {0}")]
    Cache(String),
}

impl From<CacheError> for BotError {
    fn from(error: CacheError) -> Self {
        BotError::ServiceError(ServiceError::Cache(error))
    }
}
