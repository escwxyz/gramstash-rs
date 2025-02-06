use libsql::errors::Error as TursoError;
use redis::RedisError;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Redis error: {0}")]
    Redis(String),
    #[error("Turso error: {0}")]
    Turso(#[from] TursoError),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Memory error: {0}")]
    Memory(String),
    #[error("Other error: {0}")]
    Other(String),
}

impl From<RedisError> for StorageError {
    fn from(error: RedisError) -> Self {
        StorageError::Redis(error.to_string())
    }
}
