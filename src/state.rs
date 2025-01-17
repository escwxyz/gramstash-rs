use std::sync::Arc;

use crate::{
    services::{auth::AuthService, language::LanguageService, session::SessionService},
    utils::{redis::RedisClient, turso::TursoClient},
};
use chrono::Duration;
use once_cell::sync::OnceCell;
use shuttle_runtime::SecretStore;
use tokio::sync::Mutex;

use crate::{
    config::AppConfig,
    error::{BotError, BotResult},
    services::instagram::InstagramService,
};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub redis: RedisClient,
    pub turso: TursoClient,
    pub instagram: InstagramService,
    pub auth: Arc<Mutex<AuthService>>,
    pub language: LanguageService,
    pub session: SessionService,
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
        let turso = TursoClient::new(config.turso.url.as_str(), config.turso.token.as_str()).await?;

        let instagram = InstagramService::new()?;

        let session_service =
            SessionService::with_refresh_interval(Duration::seconds(config.session.refresh_interval_secs))?;

        let auth_service = Arc::new(Mutex::new(AuthService::new()?));

        APP_STATE
            .set(AppState {
                config,
                redis,
                turso,
                instagram,
                auth: auth_service,
                session: session_service,
                language: LanguageService::new()?,
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
