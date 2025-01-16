use std::str::FromStr;

use crate::{
    command,
    error::HandlerResult,
    handlers::RequestContext,
    services::{dialogue::DialogueState, language::Language},
    state::AppState,
};
use teloxide::{dispatching::dialogue::ErasedStorage, prelude::*, types::MaybeInaccessibleMessage};

pub async fn handle_callback_language_change(
    bot: &Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    message: MaybeInaccessibleMessage,
    ctx: RequestContext,
    app_state: &AppState,
    lang_code: &str,
) -> HandlerResult<()> {
    let status_message = bot
        .send_message(message.chat().id, t!("callbacks.language.change_language_status"))
        .await?;

    let language = Language::from_str(lang_code).unwrap_or(Language::English);

    app_state
        .language
        .set_user_language(ctx.telegram_user_id.to_string(), language)
        .await?;

    rust_i18n::set_locale(&language.to_string());

    bot.edit_message_text(
        message.chat().id,
        status_message.id,
        t!(
            "callbacks.language.change_language",
            language = t!(format!("buttons.language_menu.{}", language.to_string()))
        ),
    )
    .await?;

    let return_to = app_state
        .language
        .get_last_interface(&ctx.telegram_user_id.to_string())
        .await?;

    // Update commands
    command::setup_commands(bot, ctx.is_admin, message.chat().id).await?;

    match return_to.as_str() {
        // download
        "ask_for_download_link" => {
            super::download::handle_callback_asking_for_download_link(bot, dialogue, message).await?
        }
        "confirm_download" => super::download::handle_callback_confirm_download(bot, dialogue, message).await?,
        "cancel_download" => super::download::handle_callback_cancel_download(bot, message).await?,

        // profile
        "profile_menu" | "cancel_auth" => super::profile::handle_callback_profile_menu(bot, message, ctx).await?,
        "auth_login" => super::profile::handle_callback_auth_login(bot, dialogue, message).await?,

        // navigation
        "back_to_main_menu" => super::navigation::handle_callback_back_to_main_menu(bot, dialogue, message).await?,

        _ => super::navigation::handle_callback_back_to_main_menu(bot, dialogue, message).await?,
    }

    Ok(())
}
