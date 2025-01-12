use chrono::{DateTime, Duration, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::{
    error::{BotError, BotResult, ServiceError},
    state::AppState,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionData {
    pub cookies: Vec<SerializableCookie>,
    pub user_id: Option<String>,    // ds_user_id
    pub username: Option<String>,   // we keep this for convenience
    pub csrf_token: Option<String>, // csrftoken
    pub session_id: Option<String>, // sessionid
    pub device_id: Option<String>,  // ig_did
    pub machine_id: Option<String>, // mid
    pub rur: Option<String>,        // rur
}

impl Default for SessionData {
    fn default() -> Self {
        Self {
            cookies: Vec::new(),
            user_id: None,
            username: None,
            csrf_token: None,
            session_id: None,
            device_id: None,
            machine_id: None,
            rur: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializableCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub telegram_user_id: Option<String>,
    pub session_data: Option<SessionData>,
    pub last_accessed: DateTime<Utc>,
    pub last_refresh: DateTime<Utc>,
}

impl Default for Session {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            telegram_user_id: None,
            session_data: None,
            last_accessed: now,
            last_refresh: now,
        }
    }
}

impl Session {
    #[allow(dead_code)]
    pub fn is_fresh(&self, max_age: Duration) -> bool {
        Utc::now() - self.last_refresh < max_age
    }

    #[allow(dead_code)]
    pub fn is_authenticated(&self) -> bool {
        self.session_data.is_some()
    }

    pub fn update_access(&mut self) {
        self.last_accessed = Utc::now();
    }

    pub fn update_refresh(&mut self) {
        self.last_refresh = Utc::now();
    }

    pub fn belongs_to(&self, telegram_user_id: &str) -> bool {
        self.telegram_user_id.as_deref() == Some(telegram_user_id)
    }
}

#[derive(Clone)]
pub struct SessionService {
    pub session: Session,
    refresh_interval: Duration,
}

impl SessionService {
    pub fn with_refresh_interval(refresh_interval: Duration) -> Self {
        Self {
            session: Session::default(),
            refresh_interval,
        }
    }

    pub fn needs_refresh(&self) -> bool {
        let needs_refresh = Utc::now() - self.session.last_refresh > self.refresh_interval;
        info!("Session needs refresh: {}", needs_refresh);
        needs_refresh
    }

    fn create_session_key(telegram_user_id: &str) -> String {
        format!("session:{}", telegram_user_id)
    }

    pub async fn init_telegram_user_context(&mut self, telegram_user_id: &str) -> BotResult<()> {
        if self.session.belongs_to(telegram_user_id) && !self.needs_refresh() {
            info!("Session for {} is fresh, skipping ...", telegram_user_id);
            return Ok(());
        }

        if let Some(stored_session) = self.get_session(telegram_user_id).await? {
            let mut session = stored_session;
            // Update refresh time if needed
            if self.needs_refresh() {
                info!("Refreshing session for Telegram user ID {}", telegram_user_id);
                session.last_refresh = Utc::now();
                session.last_accessed = Utc::now();
                self.upsert_session(telegram_user_id, &session).await?;
            }
            self.session = session;
        } else {
            info!("Initializing new session for Telegram user ID {}", telegram_user_id);
            self.session = Session {
                telegram_user_id: Some(telegram_user_id.to_string()),
                session_data: None,
                last_accessed: Utc::now(),
                last_refresh: Utc::now(),
            };
            let session = self.session.clone();
            self.upsert_session(telegram_user_id, &session).await?;
        }

        Ok(())
    }

    pub async fn get_session(&self, telegram_user_id: &str) -> BotResult<Option<Session>> {
        // Use local session if fresh and belongs to the same user
        if self.session.belongs_to(telegram_user_id) && !self.needs_refresh() {
            return Ok(Some(self.session.clone()));
        }

        let key = Self::create_session_key(telegram_user_id);
        let mut conn = AppState::get()?.redis.get_connection().await?;

        let session: Option<String> = conn.get(&key).await?;

        match session {
            Some(data) => {
                Ok(Some(serde_json::from_str(&data).map_err(|e| {
                    // TODO
                    BotError::ServiceError(ServiceError::Cache(e.to_string()))
                })?))
            }
            None => Ok(None),
        }
    }

    pub async fn upsert_session(&mut self, telegram_user_id: &str, session: &Session) -> BotResult<()> {
        info!("Upserting session on Redis");
        let key = Self::create_session_key(telegram_user_id);
        let state = AppState::get()?;
        let mut conn = state.redis.get_connection().await?;

        let mut session = session.clone();
        session.update_refresh();

        let serialized = serde_json::to_string(&session)
            // TODO
            .map_err(|e| BotError::ServiceError(ServiceError::Cache(format!("Failed to serialize session: {}", e))))?;
        conn.set::<_, _, String>(&key, serialized).await?;

        // Update local session
        self.session = session;

        info!("Session saved to Redis");
        Ok(())
    }

    pub async fn sync_session(&mut self, telegram_user_id: &str, session_data: SessionData) -> BotResult<()> {
        self.session.session_data = Some(session_data);
        self.session.update_access();

        // Only sync to Redis if enough time has passed
        let session = self.session.clone();
        if self.needs_refresh() {
            self.upsert_session(telegram_user_id, &session).await?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn clear_session(&mut self, telegram_user_id: &str) -> BotResult<()> {
        self.session.session_data = None;
        self.delete_session(telegram_user_id).await
    }

    #[allow(dead_code)]
    pub async fn delete_session(&self, telegram_user_id: &str) -> BotResult<()> {
        let key = Self::create_session_key(telegram_user_id);
        let state = AppState::get()?;
        let mut conn = state.redis.get_connection().await?;

        conn.del::<_, i32>(&key).await?;

        Ok(())
    }

    pub async fn validate_session(&self, telegram_user_id: &str) -> BotResult<bool> {
        if self.session.belongs_to(telegram_user_id) && !self.needs_refresh() {
            if let Some(session_data) = &self.session.session_data {
                let state = AppState::get()?;
                let mut instagram_service = state.instagram.lock().await;
                instagram_service.restore_cookies(session_data.clone())?;
                return instagram_service.verify_session().await;
            }
        }

        if let Some(stored_session) = self.get_session(telegram_user_id).await? {
            if let Some(session_data) = stored_session.session_data {
                let state = AppState::get()?;
                let mut instagram_service = state.instagram.lock().await;
                instagram_service.restore_cookies(session_data)?;
                return instagram_service.verify_session().await;
            }
        }
        Ok(false)
    }
}
