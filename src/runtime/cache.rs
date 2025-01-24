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

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Cache not found")]
    NotFound,
}

#[derive(Clone)]
pub struct CacheManager {
    memory: Option<MemoryCache<Vec<u8>>>,
    redis: Option<RedisClient>,
}

impl CacheManager {
    pub async fn new(redis_url: Option<&str>, memory_capacity: usize) -> Result<Self, StorageError> {
        let redis = if let Some(url) = redis_url {
            Some(RedisClient::new(&url).await?)
        } else {
            None
        };
        Ok(Self {
            memory: MemoryCache::new(memory_capacity),
            redis,
        })
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str, options: &CacheOptions) -> Result<Option<T>, StorageError> {
        let key = self.build_key(key, options);

        match (options.cache_type, &self.memory, &self.redis) {
            // Memory only
            (CacheType::Memory, Some(memory), _) => {
                if let Some(bytes) = memory.get(&key) {
                    Ok(serde_json::from_slice::<T>(&bytes).map(Some).unwrap())
                } else {
                    Ok(None)
                }
            }
            // Redis only
            (CacheType::Redis, _, Some(redis)) => redis.get(&key).await,
            // Both caches
            (CacheType::Both, Some(memory), Some(redis)) => {
                if let Some(bytes) = memory.get(&key) {
                    return Ok(serde_json::from_slice::<T>(&bytes).map(Some).unwrap());
                }
                if let Some(value) = redis.get(&key).await? {
                    memory.set(&key, &value);
                    return Ok(serde_json::from_slice::<T>(&value).map(Some).unwrap());
                }
                Ok(None)
            }
            // Fallback to available cache
            (CacheType::Both, Some(memory), None) => {
                if let Some(bytes) = memory.get(&key) {
                    return Ok(serde_json::from_slice::<T>(&bytes).map(Some).unwrap());
                }
                Ok(None)
            }
            (CacheType::Both, None, Some(redis)) => redis.get(&key).await,
            // No cache available
            _ => Ok(None),
        }
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: T, options: &CacheOptions) -> Result<(), StorageError> {
        let key = self.build_key(key, options);

        match (options.cache_type, &self.memory, &self.redis) {
            // Memory only
            (CacheType::Memory, Some(memory), _) => {
                let bytes = serde_json::to_vec(&value).map_err(StorageError::from)?;
                memory.set(&key, &bytes);
                Ok(())
            }
            // Redis only
            (CacheType::Redis, _, Some(redis)) => {
                let bytes = serde_json::to_vec(&value).map_err(StorageError::from)?;
                redis.set(&key, &bytes, options.ttl).await
            }
            // Both caches
            (CacheType::Both, Some(memory), Some(redis)) => {
                let bytes = serde_json::to_vec(&value).map_err(StorageError::from)?;
                memory.set(&key, &bytes);
                redis.set(&key, &bytes, options.ttl).await
            }
            // Fallback to available cache
            (CacheType::Both, Some(memory), None) => {
                let bytes = serde_json::to_vec(&value).map_err(StorageError::from)?;
                memory.set(&key, &bytes);
                Ok(())
            }
            (CacheType::Both, None, Some(redis)) => {
                let bytes = serde_json::to_vec(&value).map_err(StorageError::from)?;
                redis.set(&key, &bytes, options.ttl).await
            }
            // No cache available
            _ => Ok(()), // TODO: return error
        }
    }

    pub async fn del(&self, key: &str, options: &CacheOptions) -> Result<(), StorageError> {
        let key = self.build_key(key, options);

        match (options.cache_type, &self.memory, &self.redis) {
            // Memory only
            (CacheType::Memory, Some(memory), _) => {
                memory.del(&key);
                Ok(())
            }
            // Redis only
            (CacheType::Redis, _, Some(redis)) => redis.del(&key).await,
            // Both caches
            (CacheType::Both, Some(memory), Some(redis)) => {
                memory.del(&key);
                redis.del(&key).await
            }
            // Fallback to available cache
            (CacheType::Both, Some(memory), None) => {
                memory.del(&key);
                Ok(())
            }
            (CacheType::Both, None, Some(redis)) => redis.del(&key).await,
            // No cache available
            _ => Ok(()), // TODO: return error
        }
    }

    pub async fn keys(&self, pattern: &str, options: &CacheOptions) -> Result<Vec<String>, StorageError> {
        match (options.cache_type, &self.memory, &self.redis) {
            // Memory only
            (CacheType::Memory, Some(memory), _) => Ok(memory.keys(pattern)),
            // Redis only
            (CacheType::Redis, _, Some(redis)) => redis.keys(pattern).await,
            // Both caches
            (CacheType::Both, Some(memory), Some(redis)) => {
                let memory_keys = memory.keys(pattern);
                let redis_keys = redis.keys(pattern).await?;
                Ok(memory_keys.into_iter().chain(redis_keys).collect())
            }
            // Fallback to available cache
            (CacheType::Both, Some(memory), None) => Ok(memory.keys(pattern)),
            (CacheType::Both, None, Some(redis)) => redis.keys(pattern).await,
            // No cache available
            _ => Ok(vec![]), // TODO: return error
        }
    }

    fn build_key(&self, key: &str, options: &CacheOptions) -> String {
        if let Some(prefix) = &options.prefix {
            format!("{}:{}", prefix, key)
        } else {
            key.to_string()
        }
    }
}
