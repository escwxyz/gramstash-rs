mod download;
mod language;
mod navigation;
mod profile;

use crate::{
    error::{BotError, HandlerResult},
    services::dialogue::DialogueState,
    state::AppState,
};

use super::RequestContext;
use teloxide::{
    adaptors::Throttle,
    dispatching::{dialogue::ErasedStorage, UpdateHandler},
    prelude::*,
    types::CallbackQuery,
};

async fn handle_callback(
    bot: Throttle<Bot>,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    q: CallbackQuery,
    ctx: RequestContext,
) -> HandlerResult<()> {
    let data = q
        .data
        .ok_or_else(|| BotError::DialogueStateError("No callback data".into()))?;

    let app_state = AppState::get()?;
    let interaction = app_state.interaction;

    let message: teloxide::types::MaybeInaccessibleMessage = q
        .message
        .ok_or_else(|| BotError::DialogueStateError("No message".into()))?;

    match data.as_str() {
        // download
        "ask_for_download_link" => {
            interaction.set_last_interface(ctx.telegram_user_id.to_string(), "ask_for_download_link");
            download::handle_callback_asking_for_download_link(&bot, dialogue, message).await?
        }
        "confirm_download" => {
            interaction.set_last_interface(ctx.telegram_user_id.to_string(), "confirm_download");
            download::handle_callback_confirm_download(&bot, dialogue, message).await?
        }
        "cancel_download" => {
            interaction.set_last_interface(ctx.telegram_user_id.to_string(), "cancel_download");
            download::handle_callback_cancel_download(&bot, message).await?
        }

        // profile
        "profile_menu" => {
            interaction.set_last_interface(ctx.telegram_user_id.to_string(), "profile_menu");
            profile::handle_callback_profile_menu(&bot, message, ctx).await?
        }
        "cancel_auth" => {
            interaction.set_last_interface(ctx.telegram_user_id.to_string(), "cancel_auth");
            profile::handle_callback_profile_menu(&bot, message, ctx).await?
        }
        "auth_login" => {
            interaction.set_last_interface(ctx.telegram_user_id.to_string(), "auth_login");
            profile::handle_callback_auth_login(&bot, dialogue, message).await?
        }
        "auth_logout" => todo!(),

        // navigation
        "back_to_main_menu" => {
            interaction.set_last_interface(ctx.telegram_user_id.to_string(), "back_to_main_menu");
            navigation::handle_callback_back_to_main_menu(&bot, dialogue, message).await?
        }

        // language
        s if s.starts_with("lang:") => {
            let lang_code = s.split(":").nth(1).unwrap_or("en");
            language::handle_callback_language_change(&bot, dialogue, message, ctx, lang_code).await?
        }
        _ => todo!(),
    }

    bot.answer_callback_query(&q.id).cache_time(1).await?;

    Ok(())
}

pub fn get_callback_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    Update::filter_callback_query().endpoint(handle_callback)
}
