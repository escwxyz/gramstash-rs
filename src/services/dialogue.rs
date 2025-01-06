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
    ConfirmLogin {
        username: String,
        password: String,
    },
}

pub struct DialogueService;

impl DialogueService {
    pub async fn clear_dialogue_storage() -> BotResult<()> {
        let state = AppState::get();

        let use_redis = state.config.dialogue.use_redis;

        if !use_redis {
            return Ok(());
        }

        info!("Clearing dialogue storage...");

        let mut conn = state.redis.get_connection().await?;

        let keys: Vec<String> = conn.keys("[0-9]*").await?;

        for key in keys {
            conn.del::<_, String>(&key).await?;
            info!("Cleared dialogue state for chat_id: {}", key);
        }

        Ok(())
    }
}
