mod callback;
mod command;
mod message;

use callback::get_callback_handler;

use command::get_command_handler;
use message::{get_message_handler, handle_message_unknown};
use teloxide::{
    dispatching::{
        dialogue::{self, ErasedStorage, GetChatId},
        UpdateFilterExt, UpdateHandler,
    },
    types::{Update, UserId},
    Bot,
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
    #[allow(dead_code)]
    pub language: Language,
}

pub fn get_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    dialogue::enter::<Update, ErasedStorage<DialogueState>, DialogueState, _>()
        .filter_map_async(|update: Update, bot: Bot, state: &'static AppState| async move {
            if let Some(user) = extract_user(&update) {
                // TODO: this part does not work properly, sometimes refreshes every time
                // {
                //     let auth_service = state.auth.lock().await;
                //     let mut session_service = auth_service.session_service.clone();
                //     // TODO put init_telegram_user_context in into the middleware of the auth service
                //     if let Err(e) = session_service.init_telegram_user_context(&user.id.to_string()).await {
                //         error!("Failed to initialize telegram user context: {:?}", e);
                //     }
                // }

                let is_admin = state.config.admin.telegram_user_id == user.id;

                let language = state
                    .language
                    .get_user_language(&user.id.to_string())
                    .await
                    .unwrap_or(Language::English);

                rust_i18n::set_locale(&language.to_string());

                if is_admin {
                    if let Err(e) = crate::command::setup_admin_commands(&bot, update.chat_id().unwrap()).await {
                        error!("Failed to setup admin commands: {:?}", e);
                    }
                } else {
                    if let Err(e) = crate::command::setup_user_commands(&bot).await {
                        error!("Failed to setup user commands: {:?}", e);
                    }
                }

                let context = RequestContext {
                    telegram_user_id: user.id,
                    telegram_user_name: user.first_name,
                    is_admin,
                    language,
                };

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
