use teloxide::{
    dispatching::dialogue::ErasedStorage,
    payloads::EditMessageTextSetters,
    prelude::{Dialogue, Requester},
    types::MaybeInaccessibleMessage,
    Bot,
};

use crate::{error::HandlerResult, services::dialogue::DialogueState, utils::keyboard};

pub(super) async fn handle_callback_back_to_main_menu(
    bot: &Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    message: MaybeInaccessibleMessage,
) -> HandlerResult<()> {
    bot.edit_message_text(
        message.chat().id,
        message.id(),
        t!("callbacks.navigation.back_to_main_menu"),
    )
    .reply_markup(keyboard::MainMenu::get_inline_keyboard())
    .await?;

    dialogue.update(DialogueState::Start).await?;

    Ok(())
}
