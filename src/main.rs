use bot::BotService;
use state::AppState;
use std::{sync::Arc, time::Duration};

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod bot;
mod config;
mod handlers;
mod services;
mod state;
mod utils;

#[cfg(test)]
mod tests;

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

    let state = AppState::get();

    let bot_service = BotService::new_from_state(&state);

    info!("Bot instance created");

    Ok(bot_service)
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for BotService {
    async fn bind(self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let shared_self = Arc::new(self);

        tokio::spawn(async move {
            // interval to clear the dialogue storage
            let mut interval =
                tokio::time::interval(Duration::from_secs(AppState::get().config.dialogue.clear_interval_secs));
            loop {
                if let Err(e) = services::dialogue::DialogueService::clear_dialogue_storage().await {
                    error!("Failed to clear dialogue storage: {}", e);
                }
                interval.tick().await;
            }
        });

        shared_self.start().await.expect("Failed to start bot");

        Ok(())
    }
}
