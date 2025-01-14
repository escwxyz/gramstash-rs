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
    types::{Update, UserId},
};

use crate::{
    services::{dialogue::DialogueState, language::Language, middleware::extract_user},
    state::AppState,
};

#[derive(Clone, Debug)]
pub struct RequestContext {
    pub telegram_user_id: UserId,
    pub telegram_user_name: String,
    pub is_admin: bool,
    // TODO: think if we need to put auth service into the context
    #[allow(dead_code)]
    pub language: Language,
}

pub fn get_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    dialogue::enter::<Update, ErasedStorage<DialogueState>, DialogueState, _>()
        .filter_map_async(|update: Update, state: &'static AppState| async move {
            if let Some(user) = extract_user(&update) {
                {
                    let auth_service = state.auth.lock().await;
                    let mut session_service = auth_service.session_service.clone();
                    // TODO put init_telegram_user_context in into the middleware of the auth service
                    if let Err(e) = session_service.init_telegram_user_context(&user.id.to_string()).await {
                        error!("Failed to initialize telegram user context: {:?}", e);
                    }
                }

                let context = RequestContext {
                    telegram_user_id: user.id,
                    telegram_user_name: user.first_name,
                    is_admin: state.config.admin.telegram_user_id == user.id,
                    language: Language::English, // TODO: add language
                };

                info!("RequestContext: {:?}", context);

                Some(context) // update is always present
            } else {
                None
            }
        })
        // all handlers need the dialogue state
        .branch(get_command_handler())
        .branch(get_message_handler())
        .branch(get_callback_handler())
        .branch(Update::filter_message().endpoint(handle_message_unknown))
}
