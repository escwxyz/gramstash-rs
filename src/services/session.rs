use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::{
    state::AppState,
    utils::error::{BotError, BotResult},
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
    pub instagram_user_id: Option<String>,
    pub session_data: Option<SessionData>,
    pub last_accessed: DateTime<Utc>,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            telegram_user_id: None,
            instagram_user_id: None,
            session_data: None,
            last_accessed: Utc::now(),
        }
    }
}

#[derive(Clone)]
pub struct SessionService {
    pub session: Session,
}

impl SessionService {
    pub fn new() -> Self {
        Self {
            session: Session::default(),
        }
    }

    pub async fn init_telegram_user_context(&mut self, telegram_user_id: &str) {
        info!("Initializing session for Telegram user ID {}", telegram_user_id);
        self.session = Session {
            telegram_user_id: Some(telegram_user_id.to_string()),
            instagram_user_id: None,
            session_data: None,
            last_accessed: Utc::now(),
        };
        // Save initial session to Redis
        if let Err(e) = self.upsert_session(telegram_user_id, &self.session).await {
            error!("Failed to save initial session: {}", e);
        }
    }

    fn create_session_key(telegram_user_id: &str) -> String {
        format!("session:{}", telegram_user_id)
    }

    /// Get session from Redis
    pub async fn get_session(&self, telegram_user_id: &str) -> BotResult<Option<Session>> {
        let key = Self::create_session_key(telegram_user_id);
        let state = AppState::get()?;
        let mut conn = state.redis.get_connection().await?;

        let session: Option<String> = conn.get(&key).await?;

        match session {
            Some(data) => Ok(Some(
                serde_json::from_str(&data).map_err(|e| BotError::CacheError(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }
    // Create or update session on Redis
    pub async fn upsert_session(&self, telegram_user_id: &str, session: &Session) -> BotResult<()> {
        info!("Upserting session on Redis");
        let key = Self::create_session_key(telegram_user_id);
        let state = AppState::get()?;
        let mut conn = state.redis.get_connection().await?;

        let serialized = serde_json::to_string(session)
            .map_err(|e| BotError::CacheError(format!("Failed to serialize session: {}", e)))?;
        conn.set::<_, _, String>(&key, serialized).await?;

        Ok(())
    }
    /// Update local session, and sync with session on Redis
    pub async fn sync_session(&mut self, telegram_user_id: &str, session_data: SessionData) -> BotResult<()> {
        self.session.session_data = Some(session_data.clone());
        self.session.instagram_user_id = session_data.user_id;
        self.session.last_accessed = Utc::now();
        self.upsert_session(telegram_user_id, &self.session).await
    }

    pub async fn clear_session(&mut self, telegram_user_id: &str) -> BotResult<()> {
        // Clear local session
        self.session.session_data = None;

        // Remove from Redis
        self.delete_session(telegram_user_id).await
    }

    pub async fn delete_session(&self, telegram_user_id: &str) -> BotResult<()> {
        let key = Self::create_session_key(telegram_user_id);
        let state = AppState::get()?;
        let mut conn = state.redis.get_connection().await?;

        conn.del::<_, i32>(&key).await?;

        Ok(())
    }
}
