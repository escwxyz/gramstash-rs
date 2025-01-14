use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub status: String,
    #[allow(dead_code)]
    pub authenticated: Option<bool>,
    #[allow(dead_code)]
    pub user: Option<bool>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionData {
    pub cookies: Vec<SerializableCookie>,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub csrf_token: Option<String>,
    pub session_id: Option<String>,
    pub device_id: Option<String>,
    pub machine_id: Option<String>,
    pub rur: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    // pub language: Language, // preferred language
}

impl Default for Session {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            telegram_user_id: None,
            session_data: None,
            last_accessed: now,
            last_refresh: now,
            // language: Language::English,
        }
    }
}

impl Session {
    #[allow(dead_code)]
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
