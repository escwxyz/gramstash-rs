use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    config::AppConfig,
    error::BotResult,
    services::dialogue::{DialogueService, DialogueState},
    state::AppState,
};
use teloxide::dispatching::dialogue::ErasedStorage;

// Mutex for synchronizing test initialization
pub static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

/// Common test setup function that can be used across all test files
pub async fn setup_test_state() -> BotResult<(&'static AppState, Arc<ErasedStorage<DialogueState>>)> {
    // Lock the mutex during setup
    let _lock = TEST_MUTEX.lock().await;

    let test_config = AppConfig::new_test_config();

    // Only initialize if not already initialized
    if AppState::get().is_err() {
        AppState::init_test()
            .await
            .expect("Failed to initialize test app state");
    }

    let test_app_state = AppState::get()?;

    let storage = DialogueService::get_dialogue_storage(&test_config.dialogue)
        .await
        .expect("Failed to initialize test storage");

    Ok((test_app_state, storage))
}
