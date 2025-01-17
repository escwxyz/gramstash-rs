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
    pub is_authenticated: bool,
}

pub fn get_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    dialogue::enter::<Update, ErasedStorage<DialogueState>, DialogueState, _>()
        .filter_map_async(|update: Update, bot: Bot, state: &'static AppState| async move {
            if let Some(user) = extract_user(&update) {
                let is_admin = state.config.admin.telegram_user_id == user.id;

                let language = state
                    .language
                    .get_user_language(&user.id.to_string())
                    .await
                    .unwrap_or(Language::English);

                rust_i18n::set_locale(&language.to_string());

                if let Err(e) = crate::command::setup_commands(&bot, is_admin, update.chat_id().unwrap()).await {
                    error!("Failed to setup commands: {:?}", e);
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
