use std::sync::Arc;

use chrono::Duration;
use once_cell::sync::OnceCell;

use shuttle_runtime::SecretStore;
use tokio::sync::Mutex;

use crate::{
    config::AppConfig,
    error::{BotError, BotResult},
    services::{instagram::InstagramService, language::Language, session::SessionService},
    utils::redis::RedisClient,
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
        let config = crate::config::build_config(secret_store)?;

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
            .map_err(|_| BotError::AppStateError("App state already initialized".to_string()))?;

        Ok(())
    }

    pub fn get() -> BotResult<&'static AppState> {
        APP_STATE
            .get()
            .ok_or_else(|| BotError::AppStateError("App state not initialized".to_string()))
    }
}
