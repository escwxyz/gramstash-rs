use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use teloxide::types::UserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub telegram_user_id: UserId,
    pub telegram_user_name: String,
    pub is_admin: bool,
    pub user_tier: UserTier,
    pub created_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub total_requests: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_dialogue_state: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
pub enum UserTier {
    Subscriber = 3,
    OneTimePaid = 2,
    Free = 1,
}
