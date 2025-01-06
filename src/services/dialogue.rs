use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use teloxide::types::MessageId;

use crate::{state::AppState, utils::error::BotResult};

use super::instagram::types::MediaContent;

#[derive(Clone, Default, Serialize, Deserialize)]
pub enum DialogueState {
    #[default]
    Start,
    // Download
    AwaitingPostLink(MessageId),
    AwaitingStoryLink(MessageId),
    ConfirmDownload {
        content: MediaContent,
    },
    // Authentication
    AwaitingUsername(MessageId),
    AwaitingPassword {
        username: String,
        prompt_msg_id: MessageId,
    },
    LoggedIn,
    AwaitingLogoutConfirmation(MessageId),
    ConfirmLogout,
}

pub struct DialogueService;

impl DialogueService {
    pub async fn clear_dialogue_storage() -> BotResult<()> {
        let state = AppState::get();

        let use_redis = state.config.dialogue.use_redis;

        if !use_redis {
            debug!("Dialogue storage is not using Redis, skipping clear");
            return Ok(());
        }

        debug!("Clearing dialogue storage...");

        let mut conn = state.redis.get_connection().await?;
        debug!("Getting keys...");
        let keys: Vec<String> = conn.keys("[0-9]*").await?;

        for key in keys {
            debug!("Clearing dialogue state for chat_id: {}", key);
            conn.del::<_, String>(&key).await?;
        }

        debug!("Dialogue storage cleared");

        Ok(())
    }
}
