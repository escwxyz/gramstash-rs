mod callback;
mod command;
mod message;

use callback::get_callback_handler;

use command::get_command_handler;
use message::{get_message_handler, handle_message_unknown};
use teloxide::{
    adaptors::Throttle,
    dispatching::{
        dialogue::{self, ErasedStorage, GetChatId},
        UpdateFilterExt, UpdateHandler,
    },
    types::{Update, UserId},
    Bot,
};

use crate::{
    config::AppConfig,
    services::{dialogue::DialogueState, language::Language, middleware::extract_user},
    state::AppState,
};

#[derive(Clone, Debug)]
pub struct RequestContext {
    pub telegram_user_id: UserId,
    pub telegram_user_name: String,
    pub is_admin: bool,
    pub is_authenticated: bool,
}

pub fn get_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    dialogue::enter::<Update, ErasedStorage<DialogueState>, DialogueState, _>()
        .filter_map_async(|update: Update, bot: Throttle<Bot>| async move {
            let state = match AppState::get() {
                Ok(state) => state,
                Err(e) => {
                    error!("Failed to get AppState: {:?}", e);
                    return None;
                }
            };

            let config = match AppConfig::get() {
                Ok(config) => config,
                Err(e) => {
                    error!("Failed to get AppConfig: {:?}", e);
                    return None;
                }
            };

            if let Some(user) = extract_user(&update) {
                let is_admin = config.admin.telegram_user_id == user.id;

                let language = state
                    .language
                    .get_user_language(&user.id.to_string())
                    .await
                    .unwrap_or(Language::English);

                rust_i18n::set_locale(&language.to_string());

                if is_admin {
                    if let Err(e) = crate::command::setup_admin_commands(&bot, update.chat_id().unwrap()).await {
                        error!("Failed to setup commands: {:?}", e);
                    }
                }

                let context = RequestContext {
                    telegram_user_id: user.id,
                    telegram_user_name: user.first_name,
                    is_admin,
                    is_authenticated: state.session.is_authenticated_cached(&user.id.to_string()),
                };

                Some(context)
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
