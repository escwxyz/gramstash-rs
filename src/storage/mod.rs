mod error;
mod memory;
mod turso;
mod upstash;

pub use error::StorageError;
pub use memory::MemoryCache;
pub use turso::TursoClient;
pub use upstash::RedisClient;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

#[async_trait]
pub trait Cache: Send + Sync + 'static {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, StorageError>;
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: T,
        ttl: Option<Duration>,
    ) -> Result<(), StorageError>;
    async fn del(&self, key: &str) -> Result<(), StorageError>;
    async fn keys(&self, pattern: &str) -> Result<Vec<String>, StorageError>;
}

#[derive(Clone)]
pub struct StorageManager {
    turso: &'static TursoClient,
    redis: &'static RedisClient,
}

impl StorageManager {
    pub async fn init(redis_url: &str, turso_url: &str, turso_token: &str) -> Result<(), StorageError> {
        RedisClient::init(redis_url).await?;
        TursoClient::init(turso_url, turso_token).await?;

        Ok(())
    }

    pub async fn get() -> Result<Self, StorageError> {
        let redis = RedisClient::get()?;
        let turso = TursoClient::get()?;

        Ok(Self { redis, turso })
    }

    pub fn redis(&self) -> &RedisClient {
        self.redis
    }

    pub fn turso(&self) -> &TursoClient {
        &self.turso
    }
}
