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
    pub telegram_chat_id: Option<String>,
    pub telegram_user_id: Option<String>,
    pub instagram_user_id: Option<String>,
    pub session_data: Option<SessionData>,
    pub last_accessed: DateTime<Utc>,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            telegram_chat_id: None,
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

    pub async fn init_user_context(&mut self, telegram_chat_id: String, telegram_user_id: String) {
        self.session.telegram_chat_id = Some(telegram_chat_id);
        self.session.telegram_user_id = Some(telegram_user_id);
    }

    /// Get user context from local session
    fn get_user_context(&self) -> BotResult<(&str, &str)> {
        match (&self.session.telegram_chat_id, &self.session.telegram_user_id) {
            (Some(chat_id), Some(user_id)) => Ok((chat_id, user_id)),
            // TODO: handle this error
            _ => Err(BotError::Other(anyhow::anyhow!("User context not initialized"))),
        }
    }

    fn create_session_key(telegram_chat_id: &str, telegram_user_id: &str) -> String {
        format!("session:{}:{}", telegram_chat_id, telegram_user_id)
    }

    /// Get session from Redis
    pub async fn get_session(&self) -> BotResult<Option<Session>> {
        let (telegram_chat_id, telegram_user_id) = self.get_user_context()?;
        let key = Self::create_session_key(telegram_chat_id, telegram_user_id);
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
    pub async fn upsert_session(&self, session: &Session) -> BotResult<()> {
        let (telegram_chat_id, telegram_user_id) = self.get_user_context()?;
        let key = Self::create_session_key(telegram_chat_id, telegram_user_id);
        let state = AppState::get()?;
        let mut conn = state.redis.get_connection().await?;

        let serialized = serde_json::to_string(session)
            .map_err(|e| BotError::CacheError(format!("Failed to serialize session: {}", e)))?;
        conn.set::<_, _, String>(&key, serialized).await?;

        Ok(())
    }
    #[allow(unused)]
    pub async fn delete_session(&self) -> BotResult<()> {
        let (telegram_chat_id, telegram_user_id) = self.get_user_context()?;
        let key = Self::create_session_key(telegram_chat_id, telegram_user_id);
        let state = AppState::get()?;
        let mut conn = state.redis.get_connection().await?;

        conn.del::<_, String>(&key).await?;

        Ok(())
    }
}
