use teloxide::{
    dispatching::dialogue::ErasedStorage,
    payloads::EditMessageTextSetters,
    prelude::{Dialogue, Requester},
    types::{MaybeInaccessibleMessage, ParseMode},
    Bot,
};

use crate::{
    services::dialogue::DialogueState,
    utils::{error::HandlerResult, keyboard},
};

pub async fn handle_callback_profile_menu(bot: &Bot, message: MaybeInaccessibleMessage) -> HandlerResult<()> {
    info!("handle_callback_profile_menu");
    bot.edit_message_text(message.chat().id, message.id(), t!("callbacks.profile.profile_menu"))
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard::ProfileMenu::get_profile_menu_inline_keyboard())
        .await?;

    Ok(())
}

pub(super) async fn handle_callback_auth_login(
    bot: &Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    message: MaybeInaccessibleMessage,
) -> HandlerResult<()> {
    info!("handle_callback_auth_login");
    let username_msg = bot
        .edit_message_text(message.chat().id, message.id(), t!("callbacks.profile.auth_login"))
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard::LoginDialogue::get_cancel_auth_keyboard()) // TODO not working?
        .await?;

    dialogue
        .update(DialogueState::AwaitingUsername(username_msg.id))
        .await?;

    Ok(())
}
