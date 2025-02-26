use std::{sync::Arc, time::Duration};

use anyhow::Context;
use auth::AuthService;
use cache::CacheService;
use interaction::InteractionService;
use language::LanguageService;
use ratelimit::RateLimitService;
use session::SessionService;
use tokio::sync::Mutex;

use crate::{config::AppConfig, platform::PlatformRegistry};

mod auth;
mod cache;
pub mod dialogue;
mod error;
pub mod http;
mod interaction;
mod language;
mod ratelimit;
mod session;
mod user;

pub use auth::*;
pub use cache::Cacheable;
pub use error::ServiceError;
pub use interaction::LastInterfaceState;
pub use language::Language;
pub use session::*;

#[derive(Clone)]
pub struct ServiceRegistry {
    pub auth: Arc<Mutex<AuthService>>,
    pub session: SessionService,
    pub ratelimit: RateLimitService,
    pub language: LanguageService,
    pub interaction: InteractionService,
    pub cache: CacheService,
}

impl ServiceRegistry {
    pub async fn new(config: &AppConfig, platform_registry: Arc<PlatformRegistry>) -> Result<Self, ServiceError> {
        info!("Initializing service registry");

        let session = SessionService::new(
            Duration::from_secs(config.service.session.refresh_interval_secs),
            config.service.session.cache_capacity,
        )
        .await?;

        let ratelimit = RateLimitService::new(
            config.service.ratelimit.daily_limit,
            config.service.ratelimit.window_secs,
        )
        .await?;

        let language = LanguageService::new(config.service.language.cache_capacity).await?;

        let interaction = InteractionService::new(
            config.service.interaction.cache_capacity,
            config.service.interaction.interface_lifespan_secs,
        )
        .await
        .context("Failed to initialize interaction service")
        .unwrap();

        let cache = CacheService::new().await?;

        info!("Service registry initialized");

        Ok(Self {
            auth: Arc::new(Mutex::new(AuthService::new(platform_registry))),
            session,
            ratelimit,
            language,
            interaction,
            cache,
        })
    }
}
