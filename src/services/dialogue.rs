use std::sync::Arc;

use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::dialogue::{serializer::Json, ErasedStorage, InMemStorage, RedisStorage, Storage},
    types::MessageId,
};

use crate::{
    config::DialogueConfig,
    error::{BotError, BotResult},
    state::AppState,
};

use super::instagram::MediaContent;

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
}

pub struct DialogueService;

impl DialogueService {
    #[allow(dead_code)]
    pub async fn clear_dialogue_storage(use_redis: bool) -> BotResult<()> {
        if !use_redis {
            info!("Dialogue storage is not using Redis, skipping clear");
            return Ok(());
        }

        info!("Clearing dialogue storage...");

        let mut conn = AppState::get()?.redis.get_connection().await?;
        let keys: Vec<String> = conn.keys("[0-9]*").await?;

        for key in keys {
            info!("Clearing dialogue state for chat_id: {}", key);
            conn.del::<_, i32>(&key).await?;
        }

        info!("Dialogue storage cleared");

        Ok(())
    }

    pub async fn get_dialogue_storage(config: &DialogueConfig) -> BotResult<Arc<ErasedStorage<DialogueState>>> {
        let storage = if config.use_redis {
            let storage = {
                RedisStorage::open(config.redis_url.as_str(), Json)
                    .await
                    .map_err(|e| BotError::RedisError(e.to_string()))?
                    .erase()
            };
            storage
        } else {
            InMemStorage::new().erase()
        };

        Ok(storage)
    }
}
