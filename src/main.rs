use bot::BotService;
use config::AppConfig;
use error::BotError;
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
mod context;
mod error;
mod handler;
mod platform;
mod runtime;
mod service;
mod state;
mod storage;
mod utils;

i18n!("locales", fallback = "en");

#[shuttle_runtime::main]
async fn shuttle_main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> Result<BotService, shuttle_runtime::Error> {
    info!("Starting bot...");

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    let _ = pretty_env_logger::try_init_timed();

    info!("Initializing AppConfig...");
    AppConfig::from_env(&secrets).map_err(|e| BotError::ConfigError(e))?;
    info!("AppConfig initialized");

    let bot_service = BotService::new().await?;

    info!("Bot instance created");

    Ok(bot_service)
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for BotService {
    async fn bind(self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let shared_self = Arc::new(self);
        // let scheduler = shared_self.scheduler.clone();

        info!("Starting worker pool...");
        if let Err(e) = AppState::get()?.runtime.worker_pool.start_all().await {
            error!("Failed to start worker pool: {}", e);
            return Err(shuttle_runtime::Error::Custom(anyhow::anyhow!(e)));
        }

        shared_self
            .start()
            .await
            .map_err(|e: Box<dyn std::error::Error + Send + Sync>| {
                shuttle_runtime::Error::Custom(anyhow::anyhow!(e))
            })?;

        Ok(())
    }
}

// #[allow(dead_code)]
// async fn run_background_tasks() {
//     let state = match AppState::get() {
//         Ok(state) => state,
//         Err(e) => {
//             error!("Failed to get AppState: {}", e);
//             return;
//         }
//     };

//     let config = match AppConfig::get() {
//         Ok(config) => config,
//         Err(e) => {
//             error!("Failed to get AppConfig: {}", e);
//             return;
//         }
//     };

//     let BackgroundTasksConfig {
//         cleanup_interaction_interval_secs,
//         sync_interface_interval_secs,
//         sync_language_interval_secs,
//     } = config.background_tasks;

//     let interaction_service = state.interaction;
//     let language_service = state.language;

//     let mut cleanup_interaction_interval =
//         tokio::time::interval(Duration::from_secs(cleanup_interaction_interval_secs));
//     let mut sync_interface_interval = tokio::time::interval(Duration::from_secs(sync_interface_interval_secs));
//     let mut sync_language_interval = tokio::time::interval(Duration::from_secs(sync_language_interval_secs));

//     info!("Background tasks initialized");

//     loop {
//         tokio::select! {
//             _ = cleanup_interaction_interval.tick() => {
//                 debug!("Background task: Cleaning up old interfaces & interactions ...");
//                 interaction_service.cleanup_old_entries();
//             }
//             _ = sync_interface_interval.tick() => {
//                 debug!("Background task: Syncing interfaces to database ...");
//                 if let Err(e) = interaction_service.save_interfaces_to_database().await {
//                     error!("Failed to save interfaces to database: {}", e);
//                 }
//             }
//             _ = sync_language_interval.tick() => {
//                 debug!("Background task: Syncing languages to database ...");
//                 if let Err(e) = language_service.save_languages_to_database().await {
//                     error!("Failed to save languages to database: {}", e);
//                 }
//             }
//         }
//     }
// }
