use teloxide::{
    adaptors::Throttle,
    dispatching::dialogue::ErasedStorage,
    payloads::EditMessageTextSetters,
    prelude::{Dialogue, Requester},
    types::MaybeInaccessibleMessage,
    Bot,
};

use crate::{
    error::HandlerResult,
    handler::keyboard::{get_cancel_auth_keyboard, get_profile_menu_keyboard},
    service::dialogue::model::DialogueState,
};

pub async fn handle_callback_profile_menu(bot: &Throttle<Bot>, message: MaybeInaccessibleMessage) -> HandlerResult<()> {
    info!("handle_callback_profile_menu");
    bot.edit_message_text(message.chat().id, message.id(), t!("callbacks.profile.profile_menu"))
        .reply_markup(get_profile_menu_keyboard())
        .await?;

    Ok(())
}

pub(super) async fn handle_callback_auth_login(
    bot: &Throttle<Bot>,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    message: MaybeInaccessibleMessage,
) -> HandlerResult<()> {
    info!("handle_callback_auth_login");

    let username_msg = bot
        .edit_message_text(message.chat().id, message.id(), t!("callbacks.profile.auth_login"))
        .reply_markup(get_cancel_auth_keyboard()) // TODO not working?
        .await?;

    dialogue
        .update(DialogueState::AwaitingUsername(username_msg.id))
        .await?;

    Ok(())
}

// pub(super) async fn handle_callback_auth_logout(
//     bot: &Throttle<Bot>,
//     message: MaybeInaccessibleMessage,
// ) -> HandlerResult<()> {
//     info!("handle_callback_auth_logout");

//     bot.edit_message_text(
//         message.chat().id,
//         message.id(),
//         t!("callbacks.profile.asking_for_confirmation_on_logout"),
//     )
//     .reply_markup(get_logout_menu_keyboard())
//     .await?;

//     Ok(())
// }

// pub(super) async fn handle_callback_cancel_logout(
//     bot: &Throttle<Bot>,
//     message: MaybeInaccessibleMessage,
// ) -> HandlerResult<()> {
//     info!("handle_callback_cancel_logout");

//     bot.edit_message_text(message.chat().id, message.id(), t!("callbacks.profile.cancel_logout"))
//         .reply_markup(get_profile_menu_keyboard())
//         .await?;

//     Ok(())
// }

// pub(super) async fn handle_callback_confirm_logout(
//     bot: &Throttle<Bot>,
//     dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
//     message: MaybeInaccessibleMessage,
//     ctx: RequestContext,
// ) -> HandlerResult<()> {
//     info!("handle_callback_confirm_logout");

//     let status_msg = bot
//         .edit_message_text(
//             message.chat().id,
//             message.id(),
//             t!("callbacks.profile.confirming_logout"),
//         )
//         .await?;

//     let state = AppState::get()?;

//     let auth_service = state.service_registry.auth.lock().await;
//     let session_service = state.service_registry.session;

//     auth_service.logout(&Platform::Instagram).await?;

//     session_service
//         .remove_cached_session(&ctx.telegram_user_id.to_string(), &Platform::Instagram)
//         .await?;

//     bot.edit_message_text(message.chat().id, status_msg.id, t!("callbacks.profile.logout_success"))
//         .reply_markup(get_profile_menu_keyboard())
//         .await?;

//     dialogue.update(DialogueState::ConfirmLogout).await?;

//     Ok(())
// }
