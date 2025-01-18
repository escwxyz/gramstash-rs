use std::sync::{Arc, OnceLock};

use crate::{
    services::{
        auth::AuthService, interaction::InteractionService, language::LanguageService, session::SessionService,
    },
    utils::{redis::RedisClient, turso::TursoClient},
};
use chrono::Duration;
use tokio::sync::Mutex;

use crate::{
    config::AppConfig,
    error::{BotError, BotResult},
    services::instagram::InstagramService,
};

#[derive(Clone)]
pub struct AppState {
    pub redis: RedisClient,
    pub turso: TursoClient,
    pub instagram: InstagramService,
    pub auth: Arc<Mutex<AuthService>>,
    pub language: LanguageService,
    pub session: SessionService,
    pub interaction: InteractionService,
}

static APP_STATE: OnceLock<AppState> = OnceLock::new();

impl AppState {
    pub async fn new(config: &AppConfig) -> BotResult<Self> {
        let redis = RedisClient::new(&config.redis.url).await?;
        let turso = TursoClient::new(&config.turso.url, &config.turso.token).await?;
        let instagram = InstagramService::new()?;
        let session = SessionService::with_refresh_interval(Duration::seconds(config.session.refresh_interval_secs))?;

        let auth = Arc::new(Mutex::new(AuthService::new()?));
        let language = LanguageService::new(config.language.cache_capacity)?;

        let interaction = InteractionService::new()?;
        Ok(Self {
            redis,
            turso,
            instagram,
            auth,
            language,
            session,
            interaction,
        })
    }

    /// Initialize the global state
    pub fn set_global(state: AppState) -> BotResult<()> {
        APP_STATE
            .set(state)
            .map_err(|_| BotError::AppStateError("Failed to set global app state".into()))
    }

    /// Get the global state reference
    pub fn get() -> BotResult<AppState> {
        APP_STATE
            .get()
            .cloned()
            .ok_or_else(|| BotError::AppStateError("App state not initialized".into()))
    }
}
