use std::sync::{Arc, OnceLock};

use teloxide::adaptors::Throttle;
use teloxide::Bot;

use crate::platform::PlatformRegistry;
use crate::storage::StorageManager;
use crate::{runtime::RuntimeManager, service::ServiceRegistry};

use crate::{
    config::AppConfig,
    error::{BotError, BotResult},
};

#[derive(Clone)]
pub struct AppState {
    pub storage: StorageManager,
    pub runtime: RuntimeManager,
    pub service_registry: ServiceRegistry,
    pub platform_registry: Arc<PlatformRegistry>,
}

static APP_STATE: OnceLock<AppState> = OnceLock::new();

impl AppState {
    pub async fn new(config: &AppConfig, bot: Throttle<Bot>) -> BotResult<Self> {
        StorageManager::init(
            &config.storage.redis_url,
            &config.storage.turso_url,
            &config.storage.turso_token,
        )
        .await?;

        let storage = StorageManager::get().await?;

        let runtime = RuntimeManager::new(config.runtime.queue.capacity, bot)?;

        runtime.start().await?;

        let platform_registry = Arc::new(PlatformRegistry::new()?);

        let service_registry = ServiceRegistry::new(config, Arc::clone(&platform_registry)).await?;

        Ok(Self {
            storage,
            runtime,
            service_registry,
            platform_registry,
        })
    }

    pub fn set_global(state: AppState) -> BotResult<()> {
        APP_STATE
            .set(state)
            .map_err(|_| BotError::AppStateError("Failed to set global app state".into()))
    }

    pub fn get() -> BotResult<AppState> {
        APP_STATE
            .get()
            .cloned()
            .ok_or_else(|| BotError::AppStateError("App state not initialized".into()))
    }
}
