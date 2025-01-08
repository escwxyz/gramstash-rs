use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use teloxide::{dispatching::dialogue::ErasedStorage, prelude::Dialogue, types::MessageId};

use crate::{
    state::AppState,
    utils::error::{BotError, BotResult},
};

use super::instagram::types::MediaContent;

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub enum DialogueState {
    #[default]
    Start, // This is the first state of the dialogue
    MainMenu, // This is the main menu state
    SettingsMenu,
    HelpMenu,
    DownloadMenu,
    // Download
    AwaitingPostLink(MessageId),
    AwaitingStoryLink(MessageId),
    ConfirmDownload {
        content: MediaContent,
    },
    DownloadCancelled,
    DownloadComplete,
    RateLimitReached,
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
    // this clear all dialogue states for all users
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

    /// Reset the dialogue state to the start state
    pub async fn reset_dialogue_state(
        dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    ) -> BotResult<()> {
        dialogue
            .update(DialogueState::Start)
            .await
            .map_err(|e| BotError::DialogueError(e.to_string()))?;

        Ok(())
    }
}
