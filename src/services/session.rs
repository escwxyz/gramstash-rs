use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    config::AppConfig,
    error::{BotError, BotResult},
    state::AppState,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstagramCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionData {
    // User information from login response
    pub user_id: String,     // From login response userId
    pub username: String,    // From credentials
    pub authenticated: bool, // From login response

    // Required for authentication
    pub sessionid: InstagramCookie,  // Primary auth token
    pub ds_user_id: InstagramCookie, // User identifier

    // Required for API requests
    pub csrftoken: InstagramCookie, // Required for POST requests
    // pub rur: InstagramCookie,       // Required for routing

    // Required for device identification
    pub ig_did: InstagramCookie, // Device ID
    pub mid: InstagramCookie,    // Machine ID
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub telegram_user_id: String,
    pub session_data: Option<SessionData>,
    pub last_accessed: DateTime<Utc>,
    pub last_refresh: DateTime<Utc>,
}

#[derive(Clone)]
pub struct SessionService {
    pub session_cache: Arc<DashMap<String, Session>>,
    refresh_interval: Duration,
}

impl SessionService {
    pub fn with_refresh_interval(refresh_interval: Duration) -> BotResult<Self> {
        info!("Initializing SessionService...");
        let config = AppConfig::get()?;
        info!("SessionService initialized");
        Ok(Self {
            session_cache: Arc::new(DashMap::with_capacity(config.session.cache_capacity)),
            refresh_interval,
        })
    }

    /// Check if we need to validate the session based on cache
    fn is_session_stale(&self, telegram_user_id: &str) -> bool {
        match self.session_cache.get(telegram_user_id) {
            Some(session) => Utc::now() - session.last_refresh >= self.refresh_interval,
            None => true,
        }
    }

    /// Load session with cache-first strategy
    pub async fn get_session(&self, telegram_user_id: &str) -> BotResult<Option<Session>> {
        // Check memory cache first
        if let Some(session) = self.session_cache.get(telegram_user_id) {
            // Check if cache is still fresh
            if !self.is_session_stale(telegram_user_id) {
                return Ok(Some(session.clone()));
            }
            info!("Cache needs refresh for user: {}", telegram_user_id);
        }

        info!("No session found in cache, loading from Redis");

        // Load from Redis if not in cache or needs refresh
        if let Some(session) = self.load_session(telegram_user_id).await? {
            // Update cache
            self.session_cache.insert(telegram_user_id.to_string(), session.clone());
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    /// Load session from Redis
    async fn load_session(&self, telegram_user_id: &str) -> BotResult<Option<Session>> {
        info!("Loading session from Redis for user: {}", telegram_user_id);
        let app_state = AppState::get()?;
        let mut conn = app_state.redis.get_connection().await?;
        let key = Self::create_session_key(telegram_user_id);

        let data: Option<String> = conn.get(&key).await?;
        match data {
            Some(json) => {
                let session: Session = serde_json::from_str(&json).map_err(|e| BotError::RedisError(e.to_string()))?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    /// Get and validate session data, refreshing if necessary
    pub async fn get_valid_session(&self, telegram_user_id: &str) -> BotResult<Option<SessionData>> {
        info!("Getting valid session for user: {}", telegram_user_id);
        // Try to get session (from cache or Redis)
        if let Some(session) = self.get_session(telegram_user_id).await? {
            if let Some(session_data) = &session.session_data {
                // Check if we need to validate
                if self.is_session_stale(telegram_user_id) {
                    info!("Validating session for user: {}", telegram_user_id);
                    let state = AppState::get()?;
                    let mut auth_service = state.auth.lock().await;
                    auth_service.restore_cookies(session_data)?;

                    if auth_service.verify_session().await? {
                        info!(
                            "Session is valid, updating last_refresh and last_accessed for user: {}",
                            telegram_user_id
                        );
                        // Update last refresh time
                        let mut updated_session = session.clone();
                        updated_session.last_refresh = Utc::now();
                        updated_session.last_accessed = Utc::now();

                        // Update both cache and Redis
                        self.session_cache
                            .insert(telegram_user_id.to_string(), updated_session.clone());
                        self.save_session(telegram_user_id, session_data.clone()).await?;

                        return Ok(Some(session_data.clone()));
                    } else {
                        // Invalid session, remove from cache and Redis
                        info!(
                            "Invalid session, removing from cache and Redis for user: {}",
                            telegram_user_id
                        );
                        self.remove_session(telegram_user_id).await?;
                        return Ok(None);
                    }
                } else {
                    // Session is still fresh, just update last_accessed
                    info!(
                        "Session is still fresh, just update last_accessed for user: {}",
                        telegram_user_id
                    );
                    let mut updated_session = session.clone();
                    updated_session.last_accessed = Utc::now();
                    self.session_cache.insert(telegram_user_id.to_string(), updated_session);
                    self.save_session(telegram_user_id, session_data.clone()).await?;

                    return Ok(Some(session_data.clone()));
                }
            }
        }

        Ok(None)
    }

    pub async fn is_authenticated(&self, telegram_user_id: &str) -> BotResult<bool> {
        Ok(self.get_valid_session(telegram_user_id).await?.is_some())
    }

    pub fn is_authenticated_cached(&self, telegram_user_id: &str) -> bool {
        self.session_cache
            .get(telegram_user_id)
            .map(|session| session.session_data.is_some())
            .unwrap_or(false)
    }

    pub fn invalidate_cache(&self, telegram_user_id: &str) {
        self.session_cache.remove(telegram_user_id);
    }

    /// Save session to Redis and cache
    pub async fn save_session(&self, telegram_user_id: &str, session_data: SessionData) -> BotResult<()> {
        let session = Session {
            telegram_user_id: telegram_user_id.to_string(),
            session_data: Some(session_data),
            last_accessed: Utc::now(),
            last_refresh: Utc::now(),
        };

        let app_state = AppState::get()?;
        let mut conn = app_state.redis.get_connection().await?;
        let key = Self::create_session_key(telegram_user_id);

        let serialized = serde_json::to_string(&session).map_err(|e| BotError::RedisError(e.to_string()))?;
        conn.set::<_, _, String>(&key, serialized).await?;

        // Update cache
        self.session_cache.insert(telegram_user_id.to_string(), session);

        info!("Session saved for user: {}", telegram_user_id);

        Ok(())
    }

    /// Remove session from both cache and Redis
    async fn remove_session(&self, telegram_user_id: &str) -> BotResult<()> {
        // Remove from cache
        self.session_cache.remove(telegram_user_id);

        // Remove from Redis
        let app_state = AppState::get()?;
        let mut conn = app_state.redis.get_connection().await?;
        let key = Self::create_session_key(telegram_user_id);
        conn.del::<_, i32>(&key).await?;

        Ok(())
    }

    fn create_session_key(telegram_user_id: &str) -> String {
        format!("session:{}", telegram_user_id)
    }
}
