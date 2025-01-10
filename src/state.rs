use std::sync::Arc;

use chrono::Duration;
use once_cell::sync::OnceCell;

use shuttle_runtime::SecretStore;
use teloxide::types::UserId;
use tokio::sync::Mutex;

use crate::{
    config::{
        AdminConfig, AppConfig, CacheConfig, DialogueConfig, InstagramConfig, RateLimitConfig, RedisConfig,
        SessionConfig, TelegramConfig,
    },
    services::{instagram::InstagramService, language::Language, session::SessionService},
    utils::{
        error::{BotError, BotResult},
        redis::RedisClient,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub redis: RedisClient,
    pub instagram: Arc<Mutex<InstagramService>>,
    pub session: Arc<Mutex<SessionService>>,
    pub language: Arc<Mutex<Language>>,
}

pub static APP_STATE: OnceCell<AppState> = OnceCell::new();

impl AppState {
    pub async fn init(secret_store: &SecretStore) -> BotResult<()> {
        let config = Self::build_config(secret_store)?;

        let redis_url = config.redis.url.as_str();

        let redis = RedisClient::new(redis_url).await?;

        let instagram = Arc::new(Mutex::new(InstagramService::new()?));

        let session = Arc::new(Mutex::new(SessionService::with_refresh_interval(Duration::seconds(
            config.session.refresh_interval_secs,
        ))));

        let language = Arc::new(Mutex::new(Language::English));

        APP_STATE
            .set(AppState {
                config,
                redis,
                instagram,
                session,
                language,
            })
            .map_err(|_| anyhow::anyhow!("App state already initialized"))?;

        Ok(())
    }

    fn build_config(secret_store: &SecretStore) -> Result<AppConfig, anyhow::Error> {
        let redis_host = secret_store
            .get("UPSTASH_REDIS_HOST")
            .ok_or_else(|| anyhow::anyhow!("Missing UPSTASH_REDIS_HOST"))?;
        let redis_port = secret_store
            .get("UPSTASH_REDIS_PORT")
            .ok_or_else(|| anyhow::anyhow!("Missing UPSTASH_REDIS_PORT"))?;
        let redis_password = secret_store
            .get("UPSTASH_REDIS_PASSWORD")
            .ok_or_else(|| anyhow::anyhow!("Missing UPSTASH_REDIS_PASSWORD"))?;

        let redis_url = format!("rediss://default:{}@{}:{}", redis_password, redis_host, redis_port);

        Ok(AppConfig {
            redis: RedisConfig { url: redis_url },
            telegram: TelegramConfig(
                secret_store
                    .get("TELEGRAM_BOT_TOKEN")
                    .ok_or_else(|| anyhow::anyhow!("Missing TELEGRAM_BOT_TOKEN"))?,
            ),
            instagram: InstagramConfig {
                api_endpoint: secret_store
                    .get("INSTAGRAM_API_ENDPOINT")
                    .ok_or_else(|| anyhow::anyhow!("Missing INSTAGRAM_API_ENDPOINT"))?,
                doc_id: secret_store
                    .get("INSTAGRAM_DOC_ID")
                    .ok_or_else(|| anyhow::anyhow!("Missing INSTAGRAM_DOC_ID"))?,
            },
            rate_limit: RateLimitConfig {
                daily_limit: secret_store
                    .get("RATE_LIMIT_DAILY_LIMIT")
                    .ok_or_else(|| anyhow::anyhow!("Missing RATE_LIMIT_DAILY_LIMIT"))?
                    .parse::<u32>()
                    .map_err(|_| anyhow::anyhow!("Invalid RATE_LIMIT_DAILY_LIMIT"))?,
                window_secs: secret_store
                    .get("RATE_LIMIT_WINDOW_SECS")
                    .ok_or_else(|| anyhow::anyhow!("Missing RATE_LIMIT_WINDOW_SECS"))?
                    .parse::<u64>()
                    .map_err(|_| anyhow::anyhow!("Invalid RATE_LIMIT_WINDOW_SECS"))?,
            },
            cache: CacheConfig {
                expiry_secs: secret_store
                    .get("CACHE_EXPIRY_SECS")
                    .ok_or_else(|| anyhow::anyhow!("Missing CACHE_EXPIRY_SECS"))?
                    .parse::<u64>()
                    .map_err(|_| anyhow::anyhow!("Invalid CACHE_EXPIRY_SECS"))?,
            },
            dialogue: DialogueConfig {
                use_redis: secret_store
                    .get("DIALOGUE_USE_REDIS")
                    .ok_or_else(|| anyhow::anyhow!("Missing DIALOGUE_USE_REDIS"))?
                    .parse::<bool>()
                    .map_err(|_| anyhow::anyhow!("Invalid DIALOGUE_USE_REDIS"))?,
                clear_interval_secs: secret_store
                    .get("DIALOGUE_CLEAR_INTERVAL_SECS")
                    .ok_or_else(|| anyhow::anyhow!("Missing DIALOGUE_CLEAR_INTERVAL_SECS"))?
                    .parse::<u64>()
                    .map_err(|_| anyhow::anyhow!("Invalid DIALOGUE_CLEAR_INTERVAL_SECS"))?,
            },
            admin: AdminConfig {
                telegram_user_id: UserId(
                    secret_store
                        .get("ADMIN_TELEGRAM_USER_ID")
                        .ok_or_else(|| anyhow::anyhow!("Missing ADMIN_TELEGRAM_USER_ID"))?
                        .parse::<u64>()
                        .map_err(|_| anyhow::anyhow!("Invalid ADMIN_TELEGRAM_USER_ID"))?,
                ),
            },
            session: SessionConfig {
                refresh_interval_secs: secret_store
                    .get("SESSION_REFRESH_INTERVAL_SECS")
                    .ok_or_else(|| anyhow::anyhow!("Missing SESSION_REFRESH_INTERVAL_SECS"))?
                    .parse::<i64>()
                    .map_err(|_| anyhow::anyhow!("Invalid SESSION_REFRESH_INTERVAL_SECS"))?,
            },
        })
    }

    pub fn get() -> BotResult<&'static AppState> {
        APP_STATE
            .get()
            .ok_or_else(|| BotError::AppStateError("App state not initialized".to_string()))
    }
}
