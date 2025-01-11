use dptree;
use std::sync::Arc;
use teloxide::dispatching::dialogue::{serializer::Json, ErasedStorage, InMemStorage, RedisStorage, Storage};

use teloxide::prelude::*;
use teloxide::Bot;

use crate::error::{BotResult, HandlerResult};
use crate::handlers::get_handler;
use crate::services::dialogue::DialogueState;
use crate::state::AppState;
use crate::utils::http;

pub struct BotService {
    pub bot: Bot,
}

impl BotService {
    pub fn new_from_state(state: &AppState) -> BotResult<Self> {
        let client = http::create_telegram_client().map_err(|_| anyhow::anyhow!("Failed to create Telegram client"))?;
        Ok(Self {
            bot: Bot::with_client(state.config.telegram.0.clone(), client),
        })
    }

    pub async fn start(&self) -> HandlerResult<()> {
        let config = &AppState::get()?.config;

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
        let storage: Arc<ErasedStorage<DialogueState>> = if config.dialogue.use_redis {
            info!("Using Redis Storage");
            RedisStorage::open(config.redis.url.as_str(), Json).await?.erase()
        } else {
            info!("Using In-Memory Storage");
            InMemStorage::new().erase()
        };

        crate::command::setup_commands(&bot).await?;

        let handler = get_handler();

        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![storage]) // add deps for all below handlers?
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
