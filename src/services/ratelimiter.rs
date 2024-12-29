use anyhow::{Context, Result};
use redis::AsyncCommands;
use std::time::{SystemTime, UNIX_EPOCH};
use teloxide::types::ChatId;

use crate::state::AppState;

pub struct RateLimiter {
    max_requests: u32,
    window_seconds: u64,
}

impl RateLimiter {
    pub fn new() -> Self {
        let state = AppState::get();
        Self {
            max_requests: state.config.rate_limit.daily_limit,
            window_seconds: state.config.rate_limit.window_secs,
        }
    }

    fn generate_key(&self, chat_id: ChatId) -> String {
        let current_window = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() / self.window_seconds;

        format!("rate_limit:{}:{}", chat_id.0, current_window)
    }

    pub async fn check_rate_limit(&self, chat_id: ChatId) -> Result<bool> {
        // Block group chats
        if chat_id.0 < 0 {
            return Err(anyhow::anyhow!("Group chats are not supported"));
        }

        let key = self.generate_key(chat_id);

        let mut conn = AppState::get().redis.get_connection().await?;

        let counter: u32 = conn
            .incr::<_, u32, u32>(&key, 1)
            .await
            .context("Failed to increment rate limit counter")?;

        if counter == 1 {
            conn.expire::<_, i64>(&key, self.window_seconds as i64)
                .await
                .context("Failed to set rate limit expiry")?;
        }

        // TODO: Future enhancement - bypass for premium users
        // if is_premium_user(chat_id).await? {
        //     return Ok(true);
        // }

        Ok(counter <= self.max_requests)
    }
}
