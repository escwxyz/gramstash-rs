use redis::AsyncCommands;

use crate::{config::AppConfig, error::BotResult, state::AppState};

pub struct RateLimiter {
    max_requests: usize,
    window_seconds: u64,
}

impl RateLimiter {
    pub fn new() -> BotResult<Self> {
        let config = AppConfig::get()?;
        Ok(Self {
            max_requests: config.rate_limit.daily_limit,
            window_seconds: config.rate_limit.window_secs,
        })
    }

    pub async fn check_rate_limit(&self, telegram_user_id: &str, identifier: &str) -> BotResult<bool> {
        let mut conn = AppState::get()?.redis.get_connection().await?;

        let key = format!(
            "rate_limit:{}:{}:{}",
            telegram_user_id,
            identifier,
            chrono::Utc::now().date_naive()
        );

        let exists: bool = conn.exists(&key).await?;

        info!("Rate limit key exists: {}", exists);

        if exists {
            conn.incr::<_, _, u32>(&key, 1).await?;
            return Ok(true);
        }

        let pattern = format!("rate_limit:{}:*:{}", telegram_user_id, chrono::Utc::now().date_naive());
        let keys: Vec<String> = conn.keys(&pattern).await?;

        if keys.len() >= self.max_requests {
            info!("User has {} cached downloads", keys.len());
            return Ok(false);
        }

        conn.set_ex::<_, _, String>(&key, 1, self.window_seconds).await?;

        Ok(true)
    }
}
