use anyhow::Result;
use redis::AsyncCommands;
use std::time::{SystemTime, UNIX_EPOCH};
use teloxide::types::{ChatId, UserId};

use crate::state::AppState;

pub struct RateLimiter {
    max_requests: u32,
    window_seconds: i64,
}

impl RateLimiter {
    pub fn new() -> Self {
        let state = AppState::get();
        Self {
            max_requests: state.config.rate_limit.daily_limit,
            window_seconds: state.config.rate_limit.window_secs,
        }
    }

    // TODO: maybe chat_id alone is enough
    fn generate_key(&self, user_id: UserId, chat_id: ChatId) -> String {
        let current = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() / self.window_seconds as u64;

        format!("rate_limit:{}:{}:{}", user_id.0, chat_id.0, current)
    }

    pub async fn check_rate_limit(&self, user_id: UserId, chat_id: ChatId) -> Result<bool> {
        // TODO: allow to bypass rate limit for admins and premium users
        let key = self.generate_key(user_id, chat_id);
        let state = AppState::get();
        let mut conn = state.redis.get_connection().await?;

        let counter = conn.incr::<_, u32, u32>(&key, 1).await?;

        if counter == 1 {
            conn.expire::<_, i64>(&key, self.window_seconds).await?;
        }

        Ok(counter <= self.max_requests)
    }
}
