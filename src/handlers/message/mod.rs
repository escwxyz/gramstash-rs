pub mod download;
pub mod profile;

use teloxide::{
    dispatching::{dialogue::ErasedStorage, UpdateFilterExt, UpdateHandler},
    dptree::{self, filter},
    payloads::SendMessageSetters,
    prelude::{Dialogue, Requester},
    types::{Message, ParseMode, Update},
    Bot,
};

use crate::{
    services::dialogue::DialogueState,
    utils::{
        error::HandlerResult,
        keyboard::{self, DOWNLOAD_BUTTON, PROFILE_BUTTON},
    },
};

pub fn get_message_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    Update::filter_message()
        // IMPORTANT:first handle two persistent buttons
        .branch(
            Update::filter_message()
                .branch(
                    filter(|msg: Message| msg.text() == Some(DOWNLOAD_BUTTON))
                        .endpoint(download::handle_message_asking_for_download_link),
                )
                .branch(
                    filter(|msg: Message| msg.text() == Some(PROFILE_BUTTON))
                        .endpoint(profile::handle_message_profile_menu),
                ),
        )
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
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard::MainMenu::get_inline_keyboard())
        .await?;

    dialogue.update(DialogueState::Start).await?;
    Ok(())
}
