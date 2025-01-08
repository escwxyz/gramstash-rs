use dptree;
use std::sync::Arc;
use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::dialogue::Storage;
use teloxide::dispatching::dialogue::{serializer::Json, RedisStorage};
use teloxide::dispatching::UpdateHandler;
use teloxide::prelude::*;
use teloxide::Bot;

use crate::handlers;

use crate::handlers::command::Command;
use crate::services;
use crate::services::dialogue::DialogueState;
use crate::state::AppState;
use crate::utils::error::BotResult;
use crate::utils::error::HandlerResult;
use crate::utils::http;

pub struct BotService {
    pub bot: Bot,
}

impl BotService {
    pub fn new_from_state(state: &AppState) -> BotResult<Self> {
        info!("Initializing BotService...");
        let client = http::create_telegram_client().map_err(|_| anyhow::anyhow!("Failed to create Telegram client"))?;
        Ok(Self {
            bot: Bot::with_client(state.config.telegram.0.clone(), client),
        })
    }

    pub async fn start(&self) -> HandlerResult<()> {
        let bot = self.bot.clone();
        info!("Initializing Dialogue Storage");
        let storage: Arc<ErasedStorage<DialogueState>> = if AppState::get()?.config.dialogue.use_redis {
            info!("Using Redis Storage");
            RedisStorage::open(AppState::get()?.config.redis.url.as_str(), Json)
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

fn get_command_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    Update::filter_message()
        .filter_command::<Command>()
        .endpoint(handlers::command::command_handler)
}

fn get_message_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    Update::filter_message()
        .enter_dialogue::<Message, ErasedStorage<DialogueState>, DialogueState>()
        .branch(
            // TODO: handle this better
            dptree::filter(|msg: Message| msg.text().map(|text| text == "🏠 Main Menu").unwrap_or(false))
                .endpoint(handlers::message::handle_main_menu),
        )
        .branch(
            dptree::case![DialogueState::AwaitingPostLink(message_id)]
                .endpoint(handlers::message::download::handle_post_link),
        )
        .branch(
            dptree::case![DialogueState::AwaitingStoryLink(message_id)]
                .endpoint(handlers::message::download::handle_story_link),
        )
        .branch(
            dptree::case![DialogueState::AwaitingUsername(msg_id)].endpoint(handlers::message::login::handle_username),
        )
        .branch(
            dptree::case![DialogueState::AwaitingPassword {
                username,
                prompt_msg_id
            }]
            .endpoint(handlers::message::login::handle_password),
        )
        .branch(
            dptree::case![DialogueState::AwaitingLogoutConfirmation(msg_id)]
                .endpoint(handlers::message::logout::handle_logout),
        )
        .endpoint(handlers::message::handle_unknown_message)
}

fn get_callback_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    Update::filter_callback_query()
        .enter_dialogue::<CallbackQuery, ErasedStorage<DialogueState>, DialogueState>()
        .branch(dptree::entry().endpoint(handlers::callback::handle_callback))
}

pub fn handler_tree() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    dptree::entry()
        .filter_map_async(
            |update: Update, dialogue: Arc<ErasedStorage<DialogueState>>| async move {
                if !services::middleware::check_private_chat(&update) {
                    return None;
                }
                if let Some(telegram_user_id) = services::middleware::extract_user_id(&update) {
                    info!("Processing update for user {}", telegram_user_id);

                    // Get dialogue state
                    let _dialogue_state = if let Some(chat) = update.chat() {
                        let state = dialogue.get_dialogue(chat.id).await.ok().flatten();
                        info!("Current dialogue state: {:?}", state);
                        state
                    } else {
                        None
                    };

                    // Only handle session for None state (first interaction)
                    // TODO: this is not working as expected
                    // if dialogue_state.is_none() {
                    //     info!("No dialogue state found - initializing session");
                    //     if let Err(e) =
                    //         services::middleware::handle_user_session(&telegram_user_id, &DialogueState::Start).await
                    //     {
                    //         error!("Failed to handle user session: {}", e);
                    //     }
                    // }

                    Some(update)
                } else {
                    None
                }
            },
        )
        .branch(get_command_handler())
        .branch(get_message_handler())
        .branch(get_callback_handler())
}
