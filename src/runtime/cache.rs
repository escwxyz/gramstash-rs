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
pub struct CacheManager<T: Clone> {
    memory: Option<MemoryCache<T>>,
    redis: RedisClient,
}

impl<T> CacheManager<T>
where
    T: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    pub async fn new(redis_url: &str, memory_capacity: usize) -> Result<Self, StorageError> {
        let redis = RedisClient::new(redis_url).await?;
        Ok(Self {
            memory: MemoryCache::new(memory_capacity),
            redis,
        })
    }

    pub async fn get(&self, key: &str, options: &CacheOptions) -> Result<Option<T>, StorageError> {
        let key = self.build_key(key, options);

        match options.cache_type {
            CacheType::Memory => Ok(self.memory.as_ref().unwrap().get(&key).map(|v| v.clone())),
            CacheType::Redis => self.redis.get(&key).await,

            CacheType::Both => {
                if let Some(value) = self.memory.as_ref().unwrap().get(&key) {
                    Ok(Some(value.clone()))
                } else {
                    if let Some(value) = self.redis.get::<T>(&key).await? {
                        self.memory.as_ref().unwrap().set(&key, &value);
                        Ok(Some(value))
                    } else {
                        Ok(None)
                    }
                }
            }
        }
    }

    pub async fn set(&self, key: &str, value: T, options: &CacheOptions) -> Result<(), StorageError> {
        let key = self.build_key(key, options);

        match options.cache_type {
            CacheType::Memory => {
                self.memory.as_ref().unwrap().set(&key, &value);
                Ok(())
            }
            CacheType::Redis => self.redis.set(&key, &value, options.ttl).await,
            CacheType::Both => {
                self.memory.as_ref().unwrap().set(&key, &value);
                self.redis.set(&key, &value, options.ttl).await
            }
        }
    }

    pub async fn del(&self, key: &str, options: &CacheOptions) -> Result<(), StorageError> {
        let key = self.build_key(key, options);

        match options.cache_type {
            CacheType::Memory => {
                self.memory.as_ref().unwrap().del(&key);
                Ok(())
            }
            CacheType::Redis => self.redis.del(&key).await,
            CacheType::Both => {
                self.memory.as_ref().unwrap().del(&key);
                self.redis.del(&key).await
            }
        }
    }

    pub async fn keys(&self, pattern: &str, options: &CacheOptions) -> Result<Vec<String>, StorageError> {
        match options.cache_type {
            CacheType::Memory => Ok(self.memory.as_ref().unwrap().keys(pattern)),
            CacheType::Redis | CacheType::Both => self.redis.keys(pattern).await,
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
