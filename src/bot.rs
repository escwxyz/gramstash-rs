use dptree;
use teloxide::adaptors::throttle::Limits;
use teloxide::adaptors::Throttle;
use teloxide::prelude::*;
use teloxide::Bot;

use crate::config::AppConfig;
use crate::error::{BotResult, HandlerResult};
use crate::handlers::get_handler;
use crate::services::dialogue::DialogueService;
use crate::state::AppState;
use crate::utils::http;

pub struct BotService {
    pub bot: Throttle<Bot>,
}

impl BotService {
    pub async fn new() -> BotResult<Self> {
        info!("Initializing AppState...");
        let config = AppConfig::get()?;
        let state = AppState::new(&config).await?;
        AppState::set_global(state.clone())?;
        info!("AppState initialized");

        let builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .pool_idle_timeout(std::time::Duration::from_secs(60))
            .tcp_keepalive(std::time::Duration::from_secs(30))
            .user_agent(http::DEFAULT_USER_AGENT);

        let client = http::build_client(builder)?;

        let bot = Bot::with_client(config.telegram.0.clone(), client).throttle(Limits::default());

        Ok(Self { bot })
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
        let config = AppConfig::get()?;
        let storage = DialogueService::get_dialogue_storage(&config.dialogue).await?;

        crate::command::setup_user_commands(&bot).await?;

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
