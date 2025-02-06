use crate::storage::StorageError;

use super::{auth::AuthError, cache::CacheError, session::SessionError};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Other error: {0}")]
    Other(String),
    #[error("Auth error: {0}")]
    Auth(AuthError),
    #[error("Session error: {0}")]
    Session(SessionError),
    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),
}

impl From<AuthError> for ServiceError {
    fn from(e: AuthError) -> Self {
        Self::Auth(e)
    }
}

impl From<SessionError> for ServiceError {
    fn from(e: SessionError) -> Self {
        Self::Session(e)
    }
}

impl From<StorageError> for ServiceError {
    fn from(e: StorageError) -> Self {
        Self::Other(e.to_string())
    }
}
