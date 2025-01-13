mod callback;
mod command;
mod message;

use callback::get_callback_handler;

use command::get_command_handler;
use message::{get_message_handler, handle_message_unknown};
use teloxide::{
    dispatching::{
        dialogue::{self, ErasedStorage},
        UpdateFilterExt, UpdateHandler,
    },
    types::Update,
};

use crate::{
    services::{dialogue::DialogueState, middleware::extract_user_id},
    state::AppState,
};

pub fn get_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    dialogue::enter::<Update, ErasedStorage<DialogueState>, DialogueState, _>()
        .filter_map_async(|update: Update, state: &'static AppState| async move {
            if let Some(telegram_user_id) = extract_user_id(&update) {
                {
                    let mut session_service = state.session.lock().await;
                    if let Err(e) = session_service.init_telegram_user_context(&telegram_user_id).await {
                        error!("Failed to initialize telegram user context: {:?}", e);
                    }
                }
            }
            Some(update)
        })
        // all handlers need the dialogue state
        .branch(get_command_handler())
        .branch(get_message_handler())
        .branch(get_callback_handler())
        .branch(Update::filter_message().endpoint(handle_message_unknown))
}
