use teloxide::{
    adaptors::Throttle,
    dispatching::dialogue::ErasedStorage,
    payloads::EditMessageTextSetters,
    prelude::{Dialogue, Requester},
    types::MaybeInaccessibleMessage,
    Bot,
};

use crate::{error::HandlerResult, handlers::RequestContext, services::dialogue::DialogueState, utils::keyboard};

pub async fn handle_callback_profile_menu(
    bot: &Throttle<Bot>,
    message: MaybeInaccessibleMessage,
    ctx: RequestContext,
) -> HandlerResult<()> {
    info!("handle_callback_profile_menu");
    bot.edit_message_text(message.chat().id, message.id(), t!("callbacks.profile.profile_menu"))
        .reply_markup(keyboard::ProfileMenu::get_profile_menu_inline_keyboard(
            ctx.is_authenticated,
        ))
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
        .reply_markup(keyboard::LoginDialogue::get_cancel_auth_keyboard()) // TODO not working?
        .await?;

    dialogue
        .update(DialogueState::AwaitingUsername(username_msg.id))
        .await?;

    Ok(())
}
