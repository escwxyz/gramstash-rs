use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::{
    context::UserTier,
    platform::{DownloadState, MediaFile, Platform, PostDownloadState},
};

pub trait Task: Send + Sync + 'static {
    type Result: Send + Sync + 'static;
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct TaskContext {
    pub user_id: u64,
    pub chat_id: i64,
    pub message_id: i32,
    pub user_tier: UserTier,
    pub platform: Platform,
}

pub struct TaskWithResult<T: Task> {
    pub task: T,
    pub result_tx: oneshot::Sender<T::Result>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct DownloadTask {
    pub id: String,
    pub url: String,
    pub context: TaskContext,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Task for DownloadTask {
    type Result = DownloadState;
}

impl DownloadTask {
    pub fn new(url: String, context: TaskContext) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            url,
            context,
            created_at: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct PostDownloadTask {
    pub id: String,
    pub media_file: MediaFile,
    pub context: TaskContext,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Task for PostDownloadTask {
    type Result = PostDownloadState;
}

impl PostDownloadTask {
    pub fn new(media_file: MediaFile, context: TaskContext) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            media_file,
            context,
            created_at: chrono::Utc::now(),
        }
    }
}
