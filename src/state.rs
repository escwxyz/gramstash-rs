use std::sync::Arc;

use crate::utils::redis::RedisClient;
use chrono::Duration;
use once_cell::sync::OnceCell;
use shuttle_runtime::SecretStore;
use tokio::sync::Mutex;

use crate::{
    config::AppConfig,
    error::{BotError, BotResult},
    services::{instagram::InstagramService, language::Language, session::SessionService},
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
        Self::init_common(config).await
    }

    #[cfg(test)]
    pub async fn init_test_with_config(config: AppConfig) -> BotResult<()> {
        Self::init_common(config).await
    }

    async fn init_common(config: AppConfig) -> BotResult<()> {
        let redis = RedisClient::new(config.redis.url.as_str()).await?;

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

#[cfg(test)]
mod tests {
    use teloxide::types::UserId;

    use super::*;

    #[tokio::test]
    async fn test_app_state_init() {
        let config = AppConfig::new_test_config();

        let result = AppState::init_test_with_config(config).await;
        assert!(result.is_ok());
        let state = AppState::get().expect("App state not initialized");
        assert_eq!(state.config.admin.telegram_user_id, UserId(1234567890));
        assert_eq!(state.config.redis.url, "redis://127.0.0.1:6379".to_string());
    }
}
