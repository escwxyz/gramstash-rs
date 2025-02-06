use std::str::FromStr;

use crate::{
    command,
    context::UserContext,
    error::HandlerResult,
    platform::Platform,
    service::{dialogue::model::DialogueState, Language, LastInterfaceState},
    state::AppState,
};
use teloxide::{adaptors::Throttle, dispatching::dialogue::ErasedStorage, prelude::*, types::MaybeInaccessibleMessage};

pub async fn handle_callback_language_change(
    bot: &Throttle<Bot>,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    message: MaybeInaccessibleMessage,
    lang_code: &str,
) -> HandlerResult<()> {
    let status_message = bot
        .send_message(message.chat().id, t!("callbacks.language.change_language_status"))
        .await?;

    let language = Language::from_str(lang_code).unwrap_or(Language::English);

    let context = UserContext::global();

    let user_id = context.user_id().to_string();

    let app_state = AppState::get()?;
    app_state
        .service_registry
        .language
        .set_user_language(&user_id, language)
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

    let LastInterfaceState { interface, .. } = app_state
        .service_registry
        .interaction
        .get_last_interface(&user_id)
        .await
        .unwrap()
        .unwrap_or_default();

    let context = UserContext::global();

    // Update commands
    if context.is_admin() {
        command::setup_admin_commands(bot, message.chat().id).await?;
    } else {
        command::setup_user_commands(bot).await?;
    }

    match interface.as_str() {
        // download
        s if s.starts_with("platform:") => {
            let platform_str = s.split(":").nth(1).unwrap_or("instagram");
            let platform = Platform::from_str(platform_str)?;
            super::download::handle_callback_asking_for_download_link(bot, dialogue, message, platform).await?
        }
        // "confirm_download" => super::download::handle_callback_confirm_download(bot, dialogue, message).await?,
        "cancel_download" => super::download::handle_callback_cancel_download(bot, message).await?,

        // profile
        "profile_menu" | "cancel_auth" => super::profile::handle_callback_profile_menu(bot, message).await?,
        "auth_login" => super::profile::handle_callback_auth_login(bot, dialogue, message).await?,

        // navigation
        "back_to_main_menu" => super::navigation::handle_callback_back_to_main_menu(bot, dialogue, message).await?,

        _ => super::navigation::handle_callback_back_to_main_menu(bot, dialogue, message).await?,
    }

    Ok(())
}
