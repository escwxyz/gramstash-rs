mod memory;
mod turso;
mod upstash;

pub use memory::MemoryCache;
pub use upstash::RedisClient;

use std::time::Duration;

use async_trait::async_trait;
use libsql::errors::Error as TursoError;
use redis::RedisError;
use serde::{de::DeserializeOwned, Serialize};
use turso::TursoClient;

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
}

impl From<RedisError> for StorageError {
    fn from(error: RedisError) -> Self {
        StorageError::Redis(error.to_string())
    }
}

#[async_trait]
pub trait Cache: Send + Sync + 'static {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, StorageError>;
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<(), StorageError>;
    async fn del(&self, key: &str) -> Result<(), StorageError>;
}

#[derive(Clone)]
pub struct StorageManager {
    pub turso: TursoClient,
    pub redis: RedisClient,
}

impl StorageManager {
    pub async fn new(turso_url: &str, turso_token: &str, redis_url: &str) -> Result<Self, StorageError> {
        Ok(Self {
            turso: TursoClient::new(turso_url, turso_token).await?,
            redis: RedisClient::new(redis_url).await?,
        })
    }
}
