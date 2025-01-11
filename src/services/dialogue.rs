use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use teloxide::types::MessageId;

use crate::{error::BotResult, state::AppState};

use super::instagram::types::MediaContent;

// TODO: reduce states
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub enum DialogueState {
    #[default]
    Start,
    // Download
    AwaitingDownloadLink(MessageId),
    ConfirmDownload {
        content: MediaContent,
    },
    // Authentication
    AwaitingUsername(MessageId),
    AwaitingPassword {
        username: String,
        prompt_msg_id: MessageId,
    },
    AwaitingLogoutConfirmation(MessageId),
    ConfirmLogout,
    // Language
    // AwaitingLanguage {
    //     prompt_msg_id: MessageId,
    //     language: String,
    // },
}

pub struct DialogueService;

impl DialogueService {
    /// Clear all dialogue states for all users
    #[allow(dead_code)]
    pub async fn clear_dialogue_storage() -> BotResult<()> {
        let state = AppState::get()?;

        let use_redis = state.config.dialogue.use_redis;

        if !use_redis {
            info!("Dialogue storage is not using Redis, skipping clear");
            return Ok(());
        }

        info!("Clearing dialogue storage...");

        let mut conn = state.redis.get_connection().await?;
        let keys: Vec<String> = conn.keys("[0-9]*").await?;

        for key in keys {
            info!("Clearing dialogue state for chat_id: {}", key);
            conn.del::<_, i32>(&key).await?;
        }

        info!("Dialogue storage cleared");

        Ok(())
    }
}
