use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    platform::{instagram::InstagramSessionData, Platform},
    service::auth::AuthData,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub telegram_user_id: String,
    pub platform: Platform,

    pub status: SessionStatus,
    pub last_accessed: DateTime<Utc>,
    pub last_refresh: DateTime<Utc>,

    pub session_data: Option<SessionData>,
}

impl Session {
    pub fn get_platform_data(&self) -> Option<&PlatformSessionData> {
        self.session_data.as_ref().map(|data| &data.platform_data)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub auth_data: AuthData,
    pub platform_data: PlatformSessionData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PlatformSessionData {
    Instagram(InstagramSessionData),
    // TODO
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    Active,
    Invalid,
}
