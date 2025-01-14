use teloxide::{
    adaptors::DefaultParseMode,
    dispatching::dialogue::ErasedStorage,
    payloads::EditMessageTextSetters,
    prelude::{Dialogue, Requester},
    types::MaybeInaccessibleMessage,
    Bot,
};

use crate::{error::HandlerResult, services::dialogue::DialogueState, utils::keyboard};

pub async fn handle_callback_profile_menu(
    bot: &DefaultParseMode<Bot>,
    message: MaybeInaccessibleMessage,
) -> HandlerResult<()> {
    info!("handle_callback_profile_menu");

    // let state = AppState::get()?;

    // let auth_service = state.auth.lock().await;

    // let session_service = auth_service.session_service;

    // let telegram_user_id = session_service.session.telegram_user_id.clone().unwrap(); // TODO we don't need to obtain this from the session service

    bot.edit_message_text(message.chat().id, message.id(), t!("callbacks.profile.profile_menu"))
        .reply_markup(keyboard::ProfileMenu::get_profile_menu_inline_keyboard(
            true, // TODO: temporary
        ))
        .await?;

    Ok(())
}

pub(super) async fn handle_callback_auth_login(
    bot: &DefaultParseMode<Bot>,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    message: MaybeInaccessibleMessage,
) -> HandlerResult<()> {
    info!("handle_callback_auth_login");

    let username_msg = bot
        .edit_message_text(message.chat().id, message.id(), t!("callbacks.profile.auth_login"))
        .reply_markup(keyboard::LoginDialogue::get_cancel_auth_keyboard()) // TODO not working?
        .await?;

    dialogue
        .update(DialogueState::AwaitingUsername(username_msg.id))
        .await?;

    Ok(())
}
