use dptree;
use std::sync::Arc;
use teloxide::dispatching::dialogue::{serializer::Json, ErasedStorage, InMemStorage, RedisStorage, Storage};

use teloxide::prelude::*;
use teloxide::Bot;

use crate::handlers::{get_handler, Command};
use crate::services::dialogue::DialogueState;
use crate::state::AppState;
use crate::utils::error::{BotResult, HandlerResult};
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

        let admin_config = config.admin.clone();

        setup_commands(&bot).await?;

        let handler = get_handler();

        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![storage, admin_config])
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

pub async fn setup_commands(bot: &Bot) -> HandlerResult<()> {
    info!("Setting up bot commands...");
    match bot.set_my_commands(Command::user_commands()).await {
        Ok(_) => info!("Successfully set up bot commands"),
        Err(e) => {
            error!("Failed to set bot commands: {:?}", e);
            return Err(e.into());
        }
    }

    // TODO
    // Admin commands - set specifically for admin users
    // let admin_config = AppState::get()?.config.admin.clone();

    // // Try to set admin commands, but don't fail if we can't
    // if let Err(e) = bot
    //     .set_my_commands(Command::admin_commands())
    //     .scope(BotCommandScope::Chat {
    //         chat_id: Recipient::Id(ChatId(admin_config.telegram_user_id.0 as i64)),
    //     })
    //     .await
    // {
    //     warn!("Failed to set admin commands: {}", e);
    // }

    Ok(())
}
