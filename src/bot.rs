use dptree;
use std::time::Duration;
use teloxide::prelude::*;
use teloxide::Bot;

use crate::command::setup_user_commands;
use crate::error::{BotResult, HandlerResult};
use crate::handlers::get_handler;
use crate::services::dialogue::DialogueService;
use crate::state::AppState;
use crate::utils::http;

pub struct BotService {
    pub bot: Bot,
}

impl BotService {
    pub fn new_from_state(state: &AppState) -> BotResult<Self> {
        let builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_idle_timeout(Duration::from_secs(60))
            .tcp_keepalive(Duration::from_secs(30))
            .user_agent(http::DEFAULT_USER_AGENT);

        let client = http::build_client(builder)?;
        Ok(Self {
            bot: Bot::with_client(state.config.telegram.0.clone(), client),
        })
    }

    pub async fn start(&self) -> HandlerResult<()> {
        info!("Testing connection to Telegram API...");
        match self.bot.get_me().await {
            Ok(_) => info!("Successfully connected to Telegram API"),
            Err(e) => {
                error!("Failed to connect to Telegram API: {:?}", e);
                return Err(anyhow::anyhow!("Failed to connect to Telegram API: {}", e).into());
            }
        }

        let bot = self.bot.clone();
        let state = AppState::get()?;
        let storage = DialogueService::get_dialogue_storage(&state.config.dialogue).await?;

        setup_user_commands(&bot).await?;

        let handler = get_handler();

        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![storage, state]) // ! because of unit testing, we neeed to put AppState as a dependency for easier access in testings
            .error_handler(LoggingErrorHandler::with_custom_text(
                "An error has occurred in the dispatcher",
            ))
            .enable_ctrlc_handler()
            .build()
            // .dispatch_with_listener(update_listener, update_listener_error_handler)
            .dispatch()
            .await;

        Ok(())
    }
}
