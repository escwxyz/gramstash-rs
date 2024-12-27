use bot::BotService;
use config::Config;
use services::downloader::DownloaderService;
use std::{path::PathBuf, sync::Arc, time::Duration};
use teloxide::Bot;
use utils::{cleanup_old_files, http};

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod bot;
mod commands;
mod config;
mod handlers;
mod services;
mod utils;

#[shuttle_runtime::main]
async fn shuttle_main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> Result<BotService, shuttle_runtime::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    let _ = pretty_env_logger::try_init_timed();

    info!("Starting bot...");

    info!("Getting config...");

    let Config {
        telegram_token,
        redis_url,
        instagram_api_endpoint,
        instagram_doc_id,
    } = Config::get(&secrets);

    let client = http::create_default_client();

    // Create bot instance
    info!("Creating bot instance...");
    let bot = Bot::with_client(telegram_token, client);

    info!("Creating downloader service with redis url: {}", redis_url);

    let downloader = DownloaderService::new(
        PathBuf::from("downloads"),
        // &redis_url,
        instagram_api_endpoint,
        instagram_doc_id,
    )
    .await?;

    info!("Bot instance created");
    let bot_service = BotService { bot, downloader };

    Ok(bot_service)
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for BotService {
    async fn bind(self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let shared_self = Arc::new(self);

        start_cleanup_job().await;

        shared_self.start().await.expect("Failed to start bot");

        Ok(())
    }
}

async fn start_cleanup_job() {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60 * 60));
        loop {
            interval.tick().await;
            if let Err(e) = cleanup_old_files(PathBuf::from("downloads"), 24).await {
                log::error!("Cleanup error: {}", e);
            }
        }
    });
}
