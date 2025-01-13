use dptree;

use teloxide::adaptors::DefaultParseMode;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::Bot;

use crate::command::setup_user_commands;
use crate::error::{BotResult, HandlerResult};
use crate::handlers::get_handler;
use crate::services::dialogue::DialogueService;
use crate::state::AppState;
use crate::utils::http;

pub struct BotService {
    pub bot: DefaultParseMode<Bot>,
}

impl BotService {
    pub fn new_from_state(state: &AppState) -> BotResult<Self> {
        let client = http::create_telegram_client()?;
        Ok(Self {
            bot: Bot::with_client(state.config.telegram.0.clone(), client).parse_mode(ParseMode::Html),
        })
    }

    pub async fn start(&self) -> HandlerResult<()> {
        // Test connection before proceeding
        info!("Testing connection to Telegram API...");
        // TODO: remove this in production, use cache_me
        match self.bot.get_me().await {
            Ok(me) => info!("Successfully connected to Telegram API: {:?}", me),
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
            .dependencies(dptree::deps![storage, state]) // add deps for all below handlers?
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
