use bot::BotService;
use state::AppState;
use std::sync::Arc;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate rust_i18n;

mod bot;
mod command;
mod config;
mod handlers;
mod services;
mod state;
mod utils;

#[cfg(test)]
mod tests;

i18n!("locales");

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

    let state = AppState::get()?;

    let language = state.language.lock().await;
    let locale = language.get_locale();
    info!("Setting locale to {}", locale);
    rust_i18n::set_locale(locale);
    info!("Locale set to {}", locale);

    let bot_service = BotService::new_from_state(&state).map_err(|_| anyhow::anyhow!("Failed to create BotService"))?;

    info!("Bot instance created");

    Ok(bot_service)
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for BotService {
    async fn bind(self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let shared_self = Arc::new(self);
        // TODO: disable this for now
        // tokio::spawn(async move {
        //     let state = match AppState::get() {
        //         Ok(state) => state,
        //         Err(e) => {
        //             error!("Failed to get AppState: {}", e);
        //             return;
        //         }
        //     };

        //     let mut interval = tokio::time::interval(Duration::from_secs(state.config.dialogue.clear_interval_secs));
        //     loop {
        //         if let Err(e) = services::dialogue::DialogueService::clear_dialogue_storage().await {
        //             error!("Failed to clear dialogue storage: {}", e);
        //         }
        //         interval.tick().await;
        //     }
        // });

        shared_self
            .start()
            .await
            .map_err(|e| shuttle_runtime::Error::Custom(anyhow::anyhow!(e)))?;

        Ok(())
    }
}
