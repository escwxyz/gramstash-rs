use async_trait::async_trait;
use redis::{aio::MultiplexedConnection, AsyncCommands, Client};
use serde::{de::DeserializeOwned, Serialize};

use std::{sync::Arc, time::Duration};

use crate::storage::StorageError;

use super::Cache;

#[derive(Clone)]
pub struct RedisClient {
    inner: Arc<redis::Client>,
}

impl RedisClient {
    pub async fn new(url: &str) -> Result<Self, StorageError> {
        info!("Initializing RedisClient...");
        let redis = Arc::new(Client::open(url)?);

        let mut conn = redis.get_multiplexed_async_connection().await?;
        let pong: String = redis::cmd("PING").query_async(&mut conn).await?;
        if pong != "PONG" {
            return Err(StorageError::Redis("Redis connection test failed".to_string()));
        }
        info!("Redis connection test successful");
        info!("RedisClient initialized");
        Ok(Self { inner: redis })
    }

    pub async fn get_connection(&self) -> Result<MultiplexedConnection, StorageError> {
        let conn = self.inner.get_multiplexed_async_connection().await?;
        Ok(conn)
    }
}

#[async_trait]
impl Cache for RedisClient {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, StorageError> {
        let mut conn = self.get_connection().await?;
        let value: Option<String> = conn.get(key).await?;

        if let Some(v) = value {
            let result = serde_json::from_str(&v)?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<(), StorageError> {
        let mut conn = self.get_connection().await?;
        let serialized = serde_json::to_string(value)?;
        if let Some(ttl) = ttl {
            conn.set_ex::<_, _, String>(key, serialized, ttl.as_secs()).await?;
        } else {
            conn.set::<_, _, String>(key, serialized).await?;
        }
        Ok(())
    }

    async fn del(&self, key: &str) -> Result<(), StorageError> {
        let mut conn = self.get_connection().await?;
        conn.del::<_, i32>(key).await?;
        Ok(())
    }

    async fn keys(&self, pattern: &str) -> Result<Vec<String>, StorageError> {
        let mut conn = self.get_connection().await?;
        let keys: Vec<String> = conn.keys(pattern).await?;
        Ok(keys)
    }
}
