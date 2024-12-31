use bot::BotService;
use state::AppState;
use std::sync::Arc;
use teloxide::Bot;
use utils::http;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod bot;
mod commands;
mod config;
mod handlers;
mod services;
mod state;
mod utils;

#[shuttle_runtime::main]
async fn shuttle_main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> Result<BotService, shuttle_runtime::Error> {
    info!("Starting bot...");

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    let _ = pretty_env_logger::try_init_timed();

    info!("Initializing AppState...");
    AppState::init(&secrets).await?;

    info!("Initializing BotService...");

    let state = AppState::get();

    let client = http::create_telegram_client();

    let bot_service = BotService {
        bot: Bot::with_client(state.config.telegram.0.clone(), client),
    };

    info!("Bot instance created");

    Ok(bot_service)
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for BotService {
    async fn bind(self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let shared_self = Arc::new(self);

        // TODO
        // start_cleanup_job().await;

        shared_self.start().await.expect("Failed to start bot");

        Ok(())
    }
}

// #[allow(unused)]
// async fn start_cleanup_job() {
//     tokio::spawn(async move {
//         let mut interval = tokio::time::interval(Duration::from_secs(60 * 60));
//         loop {
//             interval.tick().await;
//             if let Err(e) = cleanup_old_files(PathBuf::from("downloads"), 24).await {
//                 log::error!("Cleanup error: {}", e);
//             }
//         }
//     });
// }
