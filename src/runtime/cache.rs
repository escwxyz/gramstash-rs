use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

use crate::storage::{Cache, MemoryCache, RedisClient, StorageError};

#[derive(Debug, Clone, Copy)]
pub enum CacheType {
    Memory,
    Redis,
    Both,
}

#[derive(Debug)]
pub struct CacheOptions {
    pub cache_type: CacheType,
    pub ttl: Option<Duration>,
    pub prefix: Option<String>,
}

impl Default for CacheOptions {
    fn default() -> Self {
        Self {
            cache_type: CacheType::Both,
            ttl: None,
            prefix: None,
        }
    }
}

#[derive(Clone)]
pub struct CacheManager {
    memory: Option<MemoryCache<String>>,
    redis: &'static RedisClient,
}

impl CacheManager {
    pub fn new(memory_capacity: usize) -> Result<Self, StorageError> {
        info!("Initializing cache manager with memory capacity: {}", memory_capacity);
        let memory = MemoryCache::new(memory_capacity);
        let redis = RedisClient::get()?;

        Ok(Self { memory, redis })
    }

    pub async fn get<T: DeserializeOwned + Serialize>(
        &self,
        key: &str,
        options: &CacheOptions,
    ) -> Result<Option<T>, StorageError> {
        let key = self.build_key(key, options);

        match (options.cache_type, &self.memory, &self.redis) {
            (CacheType::Redis, _, redis) => redis.get::<T>(&key).await,
            (CacheType::Memory, Some(memory), _) => {
                if let Some(value) = memory.get(&key) {
                    Ok(serde_json::from_str(&value)?)
                } else {
                    Ok(None)
                }
            }
            (CacheType::Both, Some(memory), redis) => {
                if let Some(value) = memory.get(&key) {
                    return Ok(serde_json::from_str(&value)?);
                }

                if let Some(value) = redis.get::<T>(&key).await? {
                    let json = serde_json::to_string(&value)?;
                    memory.set(&key, json);
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    pub async fn set<T: DeserializeOwned + Serialize + Send + Sync>(
        &self,
        key: &str,
        value: T,
        options: &CacheOptions,
    ) -> Result<(), StorageError> {
        let key = self.build_key(key, options);
        let json = serde_json::to_string(&value)?;

        match (options.cache_type, &self.memory, &self.redis) {
            (CacheType::Redis, _, redis) => {
                redis.set(&key, value, options.ttl).await?;
                Ok(())
            }
            (CacheType::Memory, Some(memory), _) => {
                memory.set(&key, json);
                Ok(())
            }
            (CacheType::Both, Some(memory), redis) => {
                memory.set(&key, json.clone());
                redis.set(&key, value, options.ttl).await?;

                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub async fn del(&self, key: &str, options: &CacheOptions) -> Result<(), StorageError> {
        let key = self.build_key(key, options);

        match (options.cache_type, &self.memory, &self.redis) {
            (CacheType::Redis, _, redis) => {
                redis.del(&key).await?;
                Ok(())
            }
            (CacheType::Memory, Some(memory), _) => {
                memory.del(&key);
                Ok(())
            }
            (CacheType::Both, Some(memory), redis) => {
                memory.del(&key);
                redis.del(&key).await?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub async fn keys(&self, pattern: &str, options: &CacheOptions) -> Result<Vec<String>, StorageError> {
        match (options.cache_type, &self.memory, &self.redis) {
            (CacheType::Memory, Some(memory), _) => Ok(memory.keys(pattern)),
            (CacheType::Redis, _, redis) => redis.keys(pattern).await,
            (CacheType::Both, Some(memory), redis) => {
                let memory_keys = memory.keys(pattern);
                let redis_keys = redis.keys(pattern).await?;
                Ok(memory_keys.into_iter().chain(redis_keys).collect())
            }

            (CacheType::Both, None, redis) => redis.keys(pattern).await,
            _ => Err(StorageError::Other("No cache available".to_string())),
        }
    }

    pub async fn ttl(&self, key: &str, options: &CacheOptions) -> Result<Option<Duration>, StorageError> {
        let key = self.build_key(key, options);
        match (options.cache_type, &self.redis) {
            (CacheType::Redis, redis) => redis.ttl(&key).await,
            (CacheType::Memory, _) => Ok(None),
            (CacheType::Both, redis) => redis.ttl(&key).await,
        }
    }

    fn build_key(&self, key: &str, options: &CacheOptions) -> String {
        if let Some(ref prefix) = options.prefix {
            format!("{}:{}", prefix, key)
        } else {
            key.to_string()
        }
    }
}
