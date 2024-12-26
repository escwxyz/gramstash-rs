use bot::BotService;
use config::Config;
use reqwest::Proxy;
use services::downloader::DownloaderService;
use std::{net::TcpStream, path::PathBuf, sync::Arc, time::Duration};
use teloxide::Bot;
use utils::cleanup_old_files;

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
    pretty_env_logger::init();

    info!("Starting bot...");

    info!("Getting config...");

    let Config {
        telegram_token,
        redis_url,
    } = Config::get(&secrets);

    // TODO: remove this after testing, this is for debugging
    #[cfg(debug_assertions)]
    let client = {
        info!("Debug mode, using proxy");
        let proxy_url = "socks5://127.0.0.1:1080";
        let proxy_addr = "127.0.0.1:1080";
        match TcpStream::connect_timeout(&proxy_addr.parse().unwrap(), Duration::from_secs(5)) {
            Ok(_) => info!("Successfully connected to proxy at {}", proxy_addr),
            Err(e) => {
                error!("Failed to connect to proxy at {}: {:?}", proxy_addr, e);
                return Err(shuttle_runtime::Error::Custom(anyhow::anyhow!(
                    "Failed to connect to proxy: {:?}",
                    e
                )));
            }
        };

        let proxy = Proxy::all(proxy_url).expect("Failed to create proxy");

        reqwest::Client::builder()
            .proxy(proxy)
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to build client")
    };

    #[cfg(not(debug_assertions))]
    let client = reqwest::Client::new();

    // Create bot instance
    info!("Creating bot instance...");
    let bot = Bot::with_client(telegram_token, client);

    let downloader = DownloaderService::new(PathBuf::from("downloads"), &redis_url).await?;

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
