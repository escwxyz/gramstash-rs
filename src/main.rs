use std::{sync::Arc, time::Duration};
use config::Config;
use redis::Client as RedisClient;
use services::downloader::DownloaderService;
use shuttle_runtime::{SecretStore, Secrets};
use teloxide::Bot;


mod handlers;
mod services;
mod utils;
mod commands;
mod config;

#[shuttle_runtime::main]
async fn shuttle_main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> Result<MyService, shuttle_runtime::Error> {


    let Config { telegram_token, redis_url } = Config::get(&secrets);

    
      // Initialize Redis client
    let redis_client = RedisClient::open(redis_url.as_str())
        .expect("Failed to create Redis client");

//     // Initialize services
    // let downloader = DownloaderService::new(redis_client.clone())
    //     .expect("Failed to create downloader service");

    // Create bot instance
    let bot = Bot::new(telegram_token);
    
    // Clone bot for command handler
    let bot_clone = bot.clone();

// Start command handler
    // tokio::spawn(async move {
    //     Command::repl(bot_clone, move |bot, msg, cmd| {
    //         let downloader = downloader.clone();
    //         async move {
    //             handle_command(bot, msg, cmd, downloader).await
    //         }
    //     })
    //     .await;
    // });

    // Ok(Bot::new("telegram-bot-token"))
    Ok(MyService {})
}

// Customize this struct with things from `shuttle_main` needed in `bind`,
// such as secrets or database connections
struct MyService {}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for MyService {
    async fn bind(self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        // Start your service and bind to the socket address
        Ok(())
    }
}


async fn start_cleanup_job(downloader: Arc<DownloaderService>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            // TODO: implement this
            // if let Err(e) = downloader.cleanup_old_files(24).await {
            //     log::error!("Cleanup error: {}", e);
            // }
        }
    });
}