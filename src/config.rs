use shuttle_runtime::SecretStore;
use std::sync::OnceLock;
use teloxide::types::UserId;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid config: {0}")]
    InvalidConfig(String),
    #[error("Load config error: {0}")]
    LoadConfigError(String),
}

// -----------------

static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub storage: StorageConfig,
    pub telegram: TelegramConfig,
    pub admin: AdminConfig,
    pub runtime: RuntimeConfig,
    pub service: ServiceConfig,
}

impl AppConfig {
    pub fn from_env(secret_store: &SecretStore) -> Result<(), ConfigError> {
        let config = Self {
            storage: StorageConfig::from_env(secret_store)?,
            telegram: TelegramConfig::from_env(secret_store)?,
            admin: AdminConfig::from_env(secret_store)?,
            runtime: RuntimeConfig::from_env(secret_store)?,
            service: ServiceConfig::from_env(secret_store)?,
        };

        let _ = APP_CONFIG
            .set(config)
            .map_err(|_| ConfigError::LoadConfigError("Failed to set global app config".to_string()));

        Ok(())
    }

    pub fn get() -> Result<&'static AppConfig, ConfigError> {
        APP_CONFIG
            .get()
            .ok_or_else(|| ConfigError::LoadConfigError("App config not initialized".to_string()))
    }
}

// -----------------

#[derive(Clone, Debug)]
pub struct StorageConfig {
    pub redis_url: String,
    pub turso_url: String,
    pub turso_token: String,
}

impl StorageConfig {
    pub fn from_env(secret_store: &SecretStore) -> Result<Self, ConfigError> {
        let redis_host = secret_store
            .get("UPSTASH_REDIS_HOST")
            .ok_or_else(|| ConfigError::LoadConfigError("Missing UPSTASH_REDIS_HOST".to_string()))?;
        let redis_port = secret_store
            .get("UPSTASH_REDIS_PORT")
            .ok_or_else(|| ConfigError::LoadConfigError("Missing UPSTASH_REDIS_PORT".to_string()))?;
        let redis_password = secret_store
            .get("UPSTASH_REDIS_PASSWORD")
            .ok_or_else(|| ConfigError::LoadConfigError("Missing UPSTASH_REDIS_PASSWORD".to_string()))?;

        let redis_url = format!("rediss://default:{}@{}:{}", redis_password, redis_host, redis_port);

        Ok(Self {
            redis_url,
            turso_url: secret_store
                .get("TURSO_URL")
                .ok_or_else(|| ConfigError::LoadConfigError("Missing TURSO_URL".to_string()))?,
            turso_token: secret_store
                .get("TURSO_TOKEN")
                .ok_or_else(|| ConfigError::LoadConfigError("Missing TURSO_TOKEN".to_string()))?,
        })
    }
}

// -----------------

#[derive(Clone, Debug)]
pub struct TelegramConfig(pub String);

impl TelegramConfig {
    pub fn from_env(secret_store: &SecretStore) -> Result<Self, ConfigError> {
        Ok(Self(secret_store.get("TELEGRAM_BOT_TOKEN").ok_or_else(|| {
            ConfigError::LoadConfigError("Missing TELEGRAM_BOT_TOKEN".to_string())
        })?))
    }
}

// -----------------

#[derive(Clone, Debug)]
pub struct AdminConfig {
    pub telegram_user_id: UserId,
}

impl AdminConfig {
    pub fn from_env(secret_store: &SecretStore) -> Result<Self, ConfigError> {
        let telegram_user_id = secret_store
            .get("ADMIN_TELEGRAM_USER_ID")
            .ok_or_else(|| ConfigError::LoadConfigError("Missing ADMIN_TELEGRAM_USER_ID".to_string()))?;

        let telegram_user_id = telegram_user_id
            .parse::<u64>()
            .map_err(|_| ConfigError::InvalidConfig("Invalid ADMIN_TELEGRAM_USER_ID".to_string()))?;

        Ok(Self {
            telegram_user_id: UserId(telegram_user_id),
        })
    }
}

// -----------------

#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    pub queue: QueueConfig,
}

impl RuntimeConfig {
    pub fn from_env(secret_store: &SecretStore) -> Result<Self, ConfigError> {
        Ok(Self {
            queue: QueueConfig {
                capacity: secret_store
                    .get("QUEUE_CAPACITY")
                    .ok_or_else(|| ConfigError::LoadConfigError("Missing QUEUE_CAPACITY".to_string()))?
                    .parse::<usize>()
                    .map_err(|_| ConfigError::InvalidConfig("Invalid QUEUE_CAPACITY".to_string()))?,
                worker_count: secret_store
                    .get("QUEUE_WORKER_COUNT")
                    .ok_or_else(|| ConfigError::LoadConfigError("Missing QUEUE_WORKER_COUNT".to_string()))?
                    .parse::<usize>()
                    .map_err(|_| ConfigError::InvalidConfig("Invalid QUEUE_WORKER_COUNT".to_string()))?,
            },
        })
    }
}

#[derive(Clone, Debug)]
pub struct QueueConfig {
    pub capacity: usize,
    #[allow(unused)]
    pub worker_count: usize,
}

// -----------------

#[derive(Clone, Debug)]
pub struct ServiceConfig {
    pub session: SessionConfig,
    pub ratelimit: RateLimitConfig,
    pub language: LanguageConfig,
    pub interaction: InteractionConfig,
    pub cache: CacheConfig,
}

impl ServiceConfig {
    pub fn from_env(secret_store: &SecretStore) -> Result<Self, ConfigError> {
        Ok(Self {
            session: SessionConfig {
                refresh_interval_secs: secret_store
                    .get("SESSION_REFRESH_INTERVAL_SECS")
                    .ok_or_else(|| ConfigError::LoadConfigError("Missing SESSION_REFRESH_INTERVAL_SECS".to_string()))?
                    .parse::<u64>()
                    .map_err(|_| ConfigError::InvalidConfig("Invalid SESSION_REFRESH_INTERVAL_SECS".to_string()))?,
                cache_capacity: secret_store
                    .get("SESSION_CACHE_CAPACITY")
                    .ok_or_else(|| ConfigError::LoadConfigError("Missing SESSION_CACHE_CAPACITY".to_string()))?
                    .parse::<usize>()
                    .map_err(|_| ConfigError::InvalidConfig("Invalid SESSION_CACHE_CAPACITY".to_string()))?,
            },
            ratelimit: RateLimitConfig {
                daily_limit: secret_store
                    .get("RATE_LIMIT_DAILY_LIMIT")
                    .ok_or_else(|| ConfigError::LoadConfigError("Missing RATE_LIMIT_DAILY_LIMIT".to_string()))?
                    .parse::<usize>()
                    .map_err(|_| ConfigError::InvalidConfig("Invalid RATE_LIMIT_DAILY_LIMIT".to_string()))?,
                window_secs: secret_store
                    .get("RATE_LIMIT_WINDOW_SECS")
                    .ok_or_else(|| ConfigError::LoadConfigError("Missing RATE_LIMIT_WINDOW_SECS".to_string()))?
                    .parse::<u64>()
                    .map_err(|_| ConfigError::InvalidConfig("Invalid RATE_LIMIT_WINDOW_SECS".to_string()))?,
            },
            language: LanguageConfig {
                cache_capacity: secret_store
                    .get("LANGUAGE_CACHE_CAPACITY")
                    .ok_or_else(|| ConfigError::LoadConfigError("Missing LANGUAGE_CACHE_CAPACITY".to_string()))?
                    .parse::<usize>()
                    .map_err(|_| ConfigError::InvalidConfig("Invalid LANGUAGE_CACHE_CAPACITY".to_string()))?,
            },
            interaction: InteractionConfig {
                cache_capacity: secret_store
                    .get("INTERACTION_CACHE_CAPACITY")
                    .ok_or_else(|| ConfigError::LoadConfigError("Missing INTERACTION_CACHE_CAPACITY".to_string()))?
                    .parse::<usize>()
                    .map_err(|_| ConfigError::InvalidConfig("Invalid INTERACTION_CACHE_CAPACITY".to_string()))?,
                interface_lifespan_secs: secret_store
                    .get("INTERACTION_INTERFACE_LIFESPAN_SECS")
                    .ok_or_else(|| {
                        ConfigError::LoadConfigError("Missing INTERACTION_INTERFACE_LIFESPAN_SECS".to_string())
                    })?
                    .parse::<i64>()
                    .map_err(|_| {
                        ConfigError::InvalidConfig("Invalid INTERACTION_INTERFACE_LIFESPAN_SECS".to_string())
                    })?,
            },
            cache: CacheConfig {
                ttl: secret_store
                    .get("CACHE_TTL")
                    .ok_or_else(|| ConfigError::LoadConfigError("Missing CACHE_TTL".to_string()))?
                    .parse::<u64>()
                    .map_err(|_| ConfigError::InvalidConfig("Invalid CACHE_TTL".to_string()))?,
            },
        })
    }
}

#[derive(Clone, Debug)]
pub struct SessionConfig {
    pub refresh_interval_secs: u64,
    /// Memory usage estimate:
    /// - Session cache: ~1KB (1009 bytes) per entry × 1,000 = ~1 MB
    /// - Actual Redis size measured: 1009 bytes per session
    /// - Additional DashMap overhead: ~16 bytes per entry
    /// Total: ~1.02 MB for 1,000 concurrent sessions
    pub cache_capacity: usize,
}

#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    pub daily_limit: usize,
    pub window_secs: u64,
}

#[derive(Clone, Debug)]
pub struct LanguageConfig {
    /// Memory usage estimate:
    /// - Language cache: ~41 bytes per entry × 20,000 = ~0.82 MB
    pub cache_capacity: usize,
}

#[derive(Clone, Debug)]
pub struct InteractionConfig {
    pub cache_capacity: usize,
    pub interface_lifespan_secs: i64,
}

#[derive(Clone, Debug)]
pub struct CacheConfig {
    pub ttl: u64,
}
