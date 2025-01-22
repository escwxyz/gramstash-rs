use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::queue::priority::Priority;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
pub enum UserTier {
    Subscriber = 3,
    OneTimePaid = 2,
    Free = 1,
}

impl From<UserTier> for Priority {
    fn from(tier: UserTier) -> Self {
        match tier {
            UserTier::Subscriber => Priority::High,
            UserTier::OneTimePaid => Priority::Normal,
            UserTier::Free => Priority::Low,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct TaskContext {
    pub user_id: i64,
    pub chat_id: i64,
    pub message_id: i32,
    pub user_tier: UserTier,
}

// TODO
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct DownloadTask {
    pub id: String,
    pub url: String,
    pub context: TaskContext,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub attempts: u32,
}

impl DownloadTask {
    pub fn new(url: String, context: TaskContext) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            url,
            context,
            created_at: chrono::Utc::now(),
            attempts: 0,
        }
    }
}

// TODO
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct CacheTask {
    pub id: String,
    pub download_task_id: String,
    pub file_id: String,
    pub context: TaskContext,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub attempts: u32,
}

impl CacheTask {
    pub fn new(download_task_id: String, file_id: String, context: TaskContext) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            download_task_id,
            file_id,
            context,
            created_at: chrono::Utc::now(),
            attempts: 0,
        }
    }
}
