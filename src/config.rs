use shuttle_runtime::SecretStore;
use teloxide::types::UserId;

use crate::error::{BotError, BotResult};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub redis: RedisConfig,
    pub telegram: TelegramConfig,
    pub instagram: InstagramConfig,
    pub rate_limit: RateLimitConfig,
    pub cache: CacheConfig,
    pub dialogue: DialogueConfig,
    pub admin: AdminConfig,
    pub session: SessionConfig,
}

impl AppConfig {
    #[cfg(test)]
    pub fn new_test_config() -> Self {
        Self {
            redis: RedisConfig::new_test_config(),
            telegram: TelegramConfig::new_test_config(),
            instagram: InstagramConfig::new_test_config(),
            rate_limit: RateLimitConfig::new_test_config(),
            cache: CacheConfig::new_test_config(),
            dialogue: DialogueConfig::new_test_config(),
            admin: AdminConfig::new_test_config(),
            session: SessionConfig::new_test_config(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RedisConfig {
    pub url: String,
}

impl RedisConfig {
    #[cfg(test)]
    pub fn new_test_config() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TelegramConfig(pub String);

impl TelegramConfig {
    #[cfg(test)]
    pub fn new_test_config() -> Self {
        Self("test_token".to_string())
    }
}

#[derive(Clone, Debug)]
pub struct InstagramConfig {
    pub api_endpoint: String,
    pub doc_id: String,
}

impl InstagramConfig {
    #[cfg(test)]
    pub fn new_test_config() -> Self {
        Self {
            api_endpoint: "https://www.instagram.com/graphql/query/".to_string(),
            doc_id: "8845758582119845".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    pub daily_limit: u32,
    pub window_secs: u64,
}

impl RateLimitConfig {
    #[cfg(test)]
    pub fn new_test_config() -> Self {
        Self {
            daily_limit: 100,
            window_secs: 86400,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CacheConfig {
    pub expiry_secs: u64,
}

impl CacheConfig {
    #[cfg(test)]
    pub fn new_test_config() -> Self {
        Self { expiry_secs: 300 }
    }
}

#[derive(Clone, Debug)]
pub struct DialogueConfig {
    pub use_redis: bool,
    pub redis_url: String,
    #[allow(dead_code)]
    pub clear_interval_secs: u64,
}

impl DialogueConfig {
    #[cfg(test)]
    pub fn new_test_config() -> Self {
        Self {
            use_redis: true,
            redis_url: "redis://127.0.0.1:6379".to_string(),
            clear_interval_secs: 60,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AdminConfig {
    pub telegram_user_id: UserId,
}

impl AdminConfig {
    #[cfg(test)]
    pub fn new_test_config() -> Self {
        Self {
            telegram_user_id: UserId(1234567890),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SessionConfig {
    pub refresh_interval_secs: i64,
}

impl SessionConfig {
    #[cfg(test)]
    pub fn new_test_config() -> Self {
        Self {
            refresh_interval_secs: 300,
        }
    }
}

pub fn build_config(secret_store: &SecretStore) -> BotResult<AppConfig> {
    let redis_host = secret_store
        .get("UPSTASH_REDIS_HOST")
        .ok_or_else(|| BotError::SecretKeyError("Missing UPSTASH_REDIS_HOST".to_string()))?;
    let redis_port = secret_store
        .get("UPSTASH_REDIS_PORT")
        .ok_or_else(|| BotError::SecretKeyError("Missing UPSTASH_REDIS_PORT".to_string()))?;
    let redis_password = secret_store
        .get("UPSTASH_REDIS_PASSWORD")
        .ok_or_else(|| BotError::SecretKeyError("Missing UPSTASH_REDIS_PASSWORD".to_string()))?;

    let redis_url = format!("rediss://default:{}@{}:{}", redis_password, redis_host, redis_port);

    Ok(AppConfig {
        redis: RedisConfig { url: redis_url.clone() },
        telegram: TelegramConfig(
            secret_store
                .get("TELEGRAM_BOT_TOKEN")
                .ok_or_else(|| BotError::SecretKeyError("Missing TELEGRAM_BOT_TOKEN".to_string()))?,
        ),
        instagram: InstagramConfig {
            api_endpoint: secret_store
                .get("INSTAGRAM_API_ENDPOINT")
                .ok_or_else(|| BotError::SecretKeyError("Missing INSTAGRAM_API_ENDPOINT".to_string()))?,
            doc_id: secret_store
                .get("INSTAGRAM_DOC_ID")
                .ok_or_else(|| BotError::SecretKeyError("Missing INSTAGRAM_DOC_ID".to_string()))?,
        },
        rate_limit: RateLimitConfig {
            daily_limit: secret_store
                .get("RATE_LIMIT_DAILY_LIMIT")
                .ok_or_else(|| BotError::SecretKeyError("Missing RATE_LIMIT_DAILY_LIMIT".to_string()))?
                .parse::<u32>()
                .map_err(|_| BotError::SecretKeyError("Invalid RATE_LIMIT_DAILY_LIMIT".to_string()))?,
            window_secs: secret_store
                .get("RATE_LIMIT_WINDOW_SECS")
                .ok_or_else(|| BotError::SecretKeyError("Missing RATE_LIMIT_WINDOW_SECS".to_string()))?
                .parse::<u64>()
                .map_err(|_| BotError::SecretKeyError("Invalid RATE_LIMIT_WINDOW_SECS".to_string()))?,
        },
        cache: CacheConfig {
            expiry_secs: secret_store
                .get("CACHE_EXPIRY_SECS")
                .ok_or_else(|| BotError::SecretKeyError("Missing CACHE_EXPIRY_SECS".to_string()))?
                .parse::<u64>()
                .map_err(|_| BotError::SecretKeyError("Invalid CACHE_EXPIRY_SECS".to_string()))?,
        },
        dialogue: DialogueConfig {
            use_redis: secret_store
                .get("DIALOGUE_USE_REDIS")
                .ok_or_else(|| BotError::SecretKeyError("Missing DIALOGUE_USE_REDIS".to_string()))?
                .parse::<bool>()
                .map_err(|_| BotError::SecretKeyError("Invalid DIALOGUE_USE_REDIS".to_string()))?,
            redis_url,
            clear_interval_secs: secret_store
                .get("DIALOGUE_CLEAR_INTERVAL_SECS")
                .ok_or_else(|| BotError::SecretKeyError("Missing DIALOGUE_CLEAR_INTERVAL_SECS".to_string()))?
                .parse::<u64>()
                .map_err(|_| BotError::SecretKeyError("Invalid DIALOGUE_CLEAR_INTERVAL_SECS".to_string()))?,
        },
        admin: AdminConfig {
            telegram_user_id: UserId(
                secret_store
                    .get("ADMIN_TELEGRAM_USER_ID")
                    .ok_or_else(|| BotError::SecretKeyError("Missing ADMIN_TELEGRAM_USER_ID".to_string()))?
                    .parse::<u64>()
                    .map_err(|_| BotError::SecretKeyError("Invalid ADMIN_TELEGRAM_USER_ID".to_string()))?,
            ),
        },
        session: SessionConfig {
            refresh_interval_secs: secret_store
                .get("SESSION_REFRESH_INTERVAL_SECS")
                .ok_or_else(|| BotError::SecretKeyError("Missing SESSION_REFRESH_INTERVAL_SECS".to_string()))?
                .parse::<i64>()
                .map_err(|_| BotError::SecretKeyError("Invalid SESSION_REFRESH_INTERVAL_SECS".to_string()))?,
        },
    })
}
