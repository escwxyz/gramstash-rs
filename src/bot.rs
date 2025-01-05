use std::sync::Arc;

use anyhow::Result;
use dptree;
use serde::Deserialize;
use serde::Serialize;
use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::dialogue::Storage;
use teloxide::dispatching::dialogue::{serializer::Json, RedisStorage};
use teloxide::dispatching::UpdateHandler;
use teloxide::prelude::*;
use teloxide::types::MessageId;
use teloxide::Bot;
use teloxide::RequestError;

use crate::handlers;

use crate::handlers::command::Command;
use crate::services;
use crate::services::instagram::types::MediaContent;
use crate::state::AppState;

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
    AwaitingUsername {
        message_id: MessageId,
        username: String,
    },
    AwaitingPassword {
        message_id: MessageId,
        username: String,
    },
}

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub struct BotService {
    pub bot: Bot,
}

impl BotService {
    pub async fn start(&self) -> HandlerResult {
        let bot = self.bot.clone();
        info!("Initializing Dialogue Storage");
        let storage: Arc<ErasedStorage<DialogueState>> = if AppState::get().config.dialogue.use_redis {
            info!("Using Redis Storage");
            RedisStorage::open(AppState::get().config.redis.url.as_str(), Json)
                .await?
                .erase()
        } else {
            info!("Using In-Memory Storage");
            InMemStorage::new().erase()
        };
        info!("Dialogue Storage initialized");

        let handler = handler_tree();

        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![storage])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}

fn get_command_handler() -> UpdateHandler<RequestError> {
    Update::filter_message()
        .filter_command::<Command>()
        .endpoint(handlers::command::command_handler)
}

fn get_message_handler() -> UpdateHandler<RequestError> {
    Update::filter_message()
        .enter_dialogue::<Message, ErasedStorage<DialogueState>, DialogueState>()
        .branch(
            dptree::case![DialogueState::Start].branch(
                dptree::filter(|msg: Message| msg.text().map(|text| text == "🏠 Main Menu").unwrap_or(false))
                    .endpoint(handlers::command::handle_start),
            ),
        )
        .branch(
            dptree::case![DialogueState::AwaitingPostLink(message_id)]
                .endpoint(handlers::message::download::handle_post_link),
        )
        .branch(
            dptree::case![DialogueState::AwaitingStoryLink(message_id)]
                .endpoint(handlers::message::download::handle_story_link),
        )
}

fn get_callback_handler() -> UpdateHandler<RequestError> {
    Update::filter_callback_query()
        .enter_dialogue::<CallbackQuery, ErasedStorage<DialogueState>, DialogueState>()
        .branch(dptree::entry().endpoint(handlers::callback::handle_callback))
}

pub fn handler_tree() -> UpdateHandler<RequestError> {
    dptree::entry()
        .filter_map_async(|update: Update| async move {
            let is_private = services::middleware::check_private_chat(&update);
            if !is_private {
                warn!("Rejected message from group chat");
                return None;
            }
            Some(update)
        })
        .branch(get_command_handler())
        .branch(get_message_handler())
        .branch(get_callback_handler())
}
