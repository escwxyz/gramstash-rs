use async_trait::async_trait;
use redis::{aio::MultiplexedConnection, AsyncCommands, Client};
use serde::{de::DeserializeOwned, Serialize};

use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};

use crate::storage::StorageError;

use super::Cache;

pub static REDIS_CLIENT: OnceLock<RedisClient> = OnceLock::new();

#[derive(Clone)]
pub struct RedisClient {
    inner: Arc<redis::Client>,
}

impl RedisClient {
    pub async fn init(url: &str) -> Result<(), StorageError> {
        if REDIS_CLIENT.get().is_some() {
            info!("Redis client already initialized");
            return Ok(());
        }

        info!("Initializing RedisClient...");
        let redis = Arc::new(Client::open(url)?);

        let mut conn = redis.get_multiplexed_async_connection().await?;
        let pong: String = redis::cmd("PING").query_async(&mut conn).await?;
        if pong != "PONG" {
            return Err(StorageError::Redis("Redis connection test failed".to_string()));
        }
        info!("Redis connection test successful");

        let client = Self { inner: redis };
        REDIS_CLIENT
            .set(client)
            .map_err(|_| StorageError::Redis("Failed to set global Redis client".into()))?;

        info!("RedisClient initialized");
        Ok(())
    }

    pub fn get() -> Result<&'static RedisClient, StorageError> {
        REDIS_CLIENT
            .get()
            .ok_or_else(|| StorageError::Redis("Redis client not initialized".into()))
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
        value: T,
        ttl: Option<Duration>,
    ) -> Result<(), StorageError> {
        let mut conn = self.get_connection().await?;
        let serialized = serde_json::to_string(&value)?;
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

    async fn ttl(&self, key: &str) -> Result<Option<Duration>, StorageError> {
        let mut conn = self.get_connection().await?;
        let ttl: Option<i64> = conn.ttl(key).await?;

        if ttl.is_none() {
            return Ok(None);
        }

        Ok(Some(Duration::from_secs(ttl.unwrap() as u64)))
    }
}
