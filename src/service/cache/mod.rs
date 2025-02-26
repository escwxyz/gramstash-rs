mod error;

pub use error::*;

use std::{sync::Arc, time::Duration};

use crate::runtime::{CacheManager, CacheOptions, CacheType};
use async_trait::async_trait;

use serde::{de::DeserializeOwned, Serialize};

#[async_trait]
pub trait Cacheable: Serialize + DeserializeOwned + Send + Sync {
    fn cache_prefix() -> &'static str;
    fn cache_key(&self) -> String;
}

/// Redis only cache service, no memory cache
#[derive(Clone)]
pub struct CacheService {
    cache: Arc<CacheManager>,
}

impl CacheService {
    pub async fn new() -> Result<Self, CacheError> {
        let cache = Arc::new(
            CacheManager::new(0).map_err(|e| CacheError::Cache(format!("Failed to create cache manager: {}", e)))?,
        );
        Ok(Self { cache })
    }

    pub async fn get<T: Cacheable>(&self, key: &str) -> Result<Option<T>, CacheError> {
        let options = CacheOptions {
            cache_type: CacheType::Redis,
            ttl: None,
            prefix: Some(T::cache_prefix().to_string()),
        };

        self.cache.get::<T>(key, &options).await.map_err(CacheError::Storage)
    }

    pub async fn set<T: Cacheable>(&self, value: T, ttl: Duration) -> Result<(), CacheError> {
        let options = CacheOptions {
            cache_type: CacheType::Redis,
            ttl: Some(ttl),
            prefix: Some(T::cache_prefix().to_string()),
        };

        self.cache
            .set(&value.cache_key(), value, &options)
            .await
            .map_err(CacheError::Storage)
    }

    // pub async fn keys<T: Cacheable>(&self, pattern: &str) -> Result<Vec<String>, CacheError> {
    //     let options = CacheOptions {
    //         cache_type: CacheType::Redis,
    //         ttl: None,
    //         prefix: Some(T::cache_prefix().to_string()),
    //     };

    //     self.cache.keys(pattern, &options).await.map_err(CacheError::Storage)
    // }

    // pub async fn delete<T: Cacheable>(&self, key: &str) -> Result<(), CacheError> {
    //     let options = CacheOptions {
    //         cache_type: CacheType::Redis,
    //         ttl: None,
    //         prefix: Some(T::cache_prefix().to_string()),
    //     };

    //     self.cache.del(key, &options).await.map_err(CacheError::Storage)
    // }
}
