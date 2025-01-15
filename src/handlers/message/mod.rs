mod download;
mod profile;

use teloxide::{
    dispatching::{dialogue::ErasedStorage, UpdateFilterExt, UpdateHandler},
    dptree::{self},
    payloads::SendMessageSetters,
    prelude::{Dialogue, Requester},
    types::{Message, Update},
    Bot,
};

use crate::{
    error::{BotError, HandlerResult},
    services::dialogue::DialogueState,
    utils::keyboard::{self},
};

pub fn get_message_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    Update::filter_message()
        // handle dialogue state
        .branch(
            dptree::case![DialogueState::AwaitingDownloadLink(message_id)]
                .endpoint(download::handle_message_awaiting_download_link),
        )
        .branch(dptree::case![DialogueState::AwaitingUsername(message_id)].endpoint(profile::handle_message_username))
        .branch(
            dptree::case![DialogueState::AwaitingPassword {
                username,
                prompt_msg_id
            }]
            .endpoint(profile::handle_message_password),
        )
}

pub async fn handle_message_unknown(
    bot: Bot,
    message: Message,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
) -> HandlerResult<()> {
    bot.delete_message(message.chat.id, message.id).await?;
    bot.send_message(message.chat.id, t!("messages.unknown_message"))
        .reply_markup(keyboard::MainMenu::get_inline_keyboard())
        .await?;

    dialogue
        .update(DialogueState::Start)
        .await
        .map_err(|e| BotError::DialogueStateError(e.to_string()))?;
    Ok(())
}
