mod download;
mod language;
mod navigation;
mod profile;

use crate::{
    error::{BotError, HandlerResult},
    services::dialogue::DialogueState,
};

use teloxide::{
    adaptors::DefaultParseMode,
    dispatching::{dialogue::ErasedStorage, UpdateHandler},
    prelude::*,
    types::CallbackQuery,
};
async fn handle_callback(
    bot: DefaultParseMode<Bot>,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    q: CallbackQuery,
) -> HandlerResult<()> {
    let data = q
        .data
        .ok_or_else(|| BotError::DialogueStateError("No callback data".into()))?;

    let message: teloxide::types::MaybeInaccessibleMessage = q
        .message
        .ok_or_else(|| BotError::DialogueStateError("No message".into()))?;

    match data.as_str() {
        // download
        "ask_for_download_link" => download::handle_callback_asking_for_download_link(&bot, dialogue, message).await?,
        "confirm_download" => download::handle_callback_confirm_download(&bot, dialogue, message).await?,
        "cancel_download" => download::handle_callback_cancel_download(&bot, message).await?,

        // profile
        "profile_menu" | "cancel_auth" => profile::handle_callback_profile_menu(&bot, message).await?,
        "auth_login" => profile::handle_callback_auth_login(&bot, dialogue, message).await?,
        "auth_logout" => todo!(),

        // language
        // "language_en" => language::handle_callback_language_en(&bot, dialogue, message).await?,
        // "language_zh" => language::handle_callback_language_zh(&bot, dialogue, message).await?,

        // navigation
        "back_to_main_menu" => navigation::handle_callback_back_to_main_menu(&bot, dialogue, message).await?,

        _ => todo!(),
    }

    bot.answer_callback_query(&q.id).await?;

    Ok(())
}

pub fn get_callback_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    Update::filter_callback_query().endpoint(handle_callback)
}
