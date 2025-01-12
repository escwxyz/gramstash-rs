use dptree;

use teloxide::prelude::*;
use teloxide::Bot;

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
        let client = http::create_telegram_client()?;
        Ok(Self {
            bot: Bot::with_client(state.config.telegram.0.clone(), client),
        })
    }

    pub async fn start(&self) -> HandlerResult<()> {
        // Test connection before proceeding
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

        crate::command::setup_commands(&bot).await?;

        let handler = get_handler();

        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![storage, state]) // add deps for all below handlers?
            .error_handler(LoggingErrorHandler::with_custom_text(
                "An error has occurred in the dispatcher",
            ))
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}
