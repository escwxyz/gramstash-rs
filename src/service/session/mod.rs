mod error;
mod model;
pub use error::SessionError;

use async_trait::async_trait;
use chrono::Utc;
use std::time::Duration;

use crate::{
    platform::{instagram::PlatformInstagram, traits::PlatformCapability, Platform},
    runtime::{CacheManager, CacheOptions, CacheType},
    state::AppState,
};

pub use model::*;

#[derive(Clone)]
pub struct SessionService {
    cache: CacheManager,
    refresh_interval: Duration,
}

#[async_trait]
pub trait PlatformSession: PlatformCapability {
    async fn validate_session(&self, session: &Session) -> Result<SessionStatus, SessionError>;
}

impl SessionService {
    pub async fn new(refresh_interval: Duration, cache_capacity: usize) -> Result<Self, SessionError> {
        info!("Initializing session service");
        info!("Session service initialized");

        let cache = CacheManager::new(cache_capacity).unwrap();

        Ok(Self {
            cache,
            refresh_interval,
        })
    }

    fn build_cache_options(&self, platform: &Platform) -> CacheOptions {
        CacheOptions {
            cache_type: CacheType::Both,
            ttl: Some(self.refresh_interval),
            prefix: Some(format!("session:{}:", platform.to_string().to_lowercase())),
        }
    }

    pub async fn get_cached_session(
        &self,
        telegram_user_id: &str,
        platform: &Platform,
    ) -> Result<Option<Session>, SessionError> {
        let options = self.build_cache_options(platform);

        self.cache
            .get::<Session>(telegram_user_id, &options)
            .await
            .map_err(|e| SessionError::CacheError(e.to_string()))
    }

    pub async fn save_cached_session(&self, session: Session, platform: &Platform) -> Result<(), SessionError> {
        let options = self.build_cache_options(platform);

        self.cache
            .set::<Session>(session.telegram_user_id.as_str(), session.clone(), &options)
            .await
            .map_err(|e| SessionError::CacheError(e.to_string()))
    }

    pub async fn remove_cached_session(&self, telegram_user_id: &str, platform: &Platform) -> Result<(), SessionError> {
        let options = self.build_cache_options(platform);

        self.cache
            .del(telegram_user_id, &options)
            .await
            .map_err(|e| SessionError::CacheError(e.to_string()))
    }
    // TODO: generic platform session
    pub async fn get_valid_session(
        &self,
        telegram_user_id: &str,
        platform: &Platform,
    ) -> Result<Option<Session>, SessionError> {
        if let Some(mut session) = self.get_cached_session(telegram_user_id, platform).await? {
            session.last_accessed = Utc::now();

            if Utc::now() - session.last_refresh >= chrono::Duration::from_std(self.refresh_interval).unwrap() {
                // Currently only Instagram is supported
                let platform_service = AppState::get()
                    .unwrap()
                    .platform_registry
                    .get_platform::<PlatformInstagram>(platform)
                    .unwrap();

                let platform_session = platform_service
                    .as_any()
                    .downcast_ref::<Box<dyn PlatformSession>>()
                    .ok_or(SessionError::SessionNotFound)?;
                session.status = platform_session.validate_session(&session).await?;

                match session.status {
                    SessionStatus::Active => {
                        session.last_refresh = Utc::now();
                        self.save_cached_session(session.clone(), platform).await?;

                        Ok(Some(session))
                    }
                    _ => {
                        self.remove_cached_session(telegram_user_id, platform).await?;
                        Ok(None)
                    }
                }
            } else {
                Ok(Some(session))
            }
        } else {
            Ok(None)
        }
    }

    pub async fn is_authenticated(&self, telegram_user_id: &str, platform: &Platform) -> Result<bool, SessionError> {
        Ok(self.get_valid_session(telegram_user_id, platform).await?.is_some())
    }

    // pub async fn is_authenticated_cached(
    //     &self,
    //     telegram_user_id: &str,
    //     platform: &Platform,
    // ) -> Result<bool, SessionError> {
    //     Ok(self.get_cached_session(telegram_user_id, platform).await?.is_some())
    // }
}
