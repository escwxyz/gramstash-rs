use teloxide::{
    adaptors::Throttle,
    dispatching::dialogue::ErasedStorage,
    payloads::EditMessageTextSetters,
    prelude::{Dialogue, Requester},
    types::MaybeInaccessibleMessage,
    Bot,
};

use crate::{error::HandlerResult, handler::keyboard::get_main_menu_keyboard, service::dialogue::model::DialogueState};

pub(super) async fn handle_callback_back_to_main_menu(
    bot: &Throttle<Bot>,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    message: MaybeInaccessibleMessage,
) -> HandlerResult<()> {
    bot.edit_message_text(
        message.chat().id,
        message.id(),
        t!("callbacks.navigation.back_to_main_menu"),
    )
    .reply_markup(get_main_menu_keyboard())
    .await?;

    dialogue.update(DialogueState::Start).await?;

    Ok(())
}
