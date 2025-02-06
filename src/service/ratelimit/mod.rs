mod model;

pub use model::RateLimitInfo;

use chrono::Utc;
use std::time::Duration;

use crate::{
    runtime::{CacheManager, CacheOptions, CacheType},
    storage::StorageError,
    utils::seconds_to_human_readable,
};

#[derive(Clone)]
pub struct RateLimitService {
    cache: CacheManager,
    max_requests: usize,
    window_seconds: Duration,
}

impl RateLimitService {
    pub async fn new(daily_limit: usize, window_secs: u64) -> Result<Self, StorageError> {
        info!("Initializing rate limit service");
        Ok(Self {
            cache: CacheManager::new(0)?,
            max_requests: daily_limit,
            window_seconds: Duration::from_secs(window_secs),
        })
    }

    pub async fn check_rate_limit(&self, telegram_user_id: &str, identifier: &str) -> Result<bool, StorageError> {
        let today = Utc::now().date_naive();
        let key = format!("rate_limit:{}:{}:{}", telegram_user_id, identifier, today.to_string());

        info!("key: {}", key);

        let options = CacheOptions {
            cache_type: CacheType::Redis,
            ttl: Some(self.window_seconds),
            prefix: None,
        };

        if let Some(count) = self.cache.get::<u32>(&key, &options).await? {
            info!("count: {}", count);

            self.cache.set::<u32>(&key, count + 1, &options).await?;
            return Ok(true);
        }

        let pattern = format!("rate_limit:{}:*:{}", telegram_user_id, today);
        let keys = self.cache.keys(&pattern, &options).await?;

        info!("keys: {:?}", keys);

        if keys.len() >= self.max_requests {
            return Ok(false);
        }

        self.cache.set::<u32>(&key, 1, &options).await?;
        Ok(true)
    }

    pub async fn get_rate_limit_info(&self, telegram_user_id: &str) -> Result<RateLimitInfo, StorageError> {
        let today = Utc::now().date_naive();
        let options = CacheOptions {
            cache_type: CacheType::Redis,
            ttl: Some(self.window_seconds),
            prefix: None,
        };
        let pattern = format!("rate_limit:{}:*:{}", telegram_user_id, today);
        let keys = self.cache.keys(&pattern, &options).await?;

        if keys.is_empty() {
            return Ok(RateLimitInfo {
                total_requests: 0,
                total_used_requests: 0,
                remaining_requests: self.max_requests,
                reset_time: seconds_to_human_readable(self.window_seconds.as_secs()),
            });
        }

        let mut total_requests = 0; // combined request number to all resources today
        let total_used_requests = keys.len(); // total requests to different resources used today

        let mut max_ttl = Duration::from_secs(0);

        for key in keys {
            if let Some(count) = self.cache.get::<u32>(&key, &options).await? {
                total_requests += count;
                if let Some(ttl) = self.cache.ttl(&key, &options).await? {
                    max_ttl = max_ttl.max(ttl);
                }
            }
        }

        let remaining_requests = self.max_requests - total_used_requests;
        let reset_time = seconds_to_human_readable(max_ttl.as_secs());

        Ok(RateLimitInfo {
            total_requests,
            total_used_requests,
            remaining_requests,
            reset_time,
        })
    }
}
