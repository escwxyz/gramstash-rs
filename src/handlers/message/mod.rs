mod download;
mod profile;

use teloxide::{
    dispatching::{UpdateFilterExt, UpdateHandler},
    dptree::{self, filter},
    types::{Message, Update},
};

use crate::{services::dialogue::DialogueState, utils::keyboard::DOWNLOAD_BUTTON, utils::keyboard::PROFILE_BUTTON};

use super::callback::handle_callback_profile_menu;

// Only handle message based on dialogue state
pub fn get_message_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    Update::filter_message()
        // handle two persistent buttons
        .branch(
            dptree::case![DialogueState::Start].branch(
                // why shall we in start state?
                Update::filter_message().branch(
                    filter(|msg: Message| msg.text() == Some(DOWNLOAD_BUTTON))
                        .endpoint(download::handle_message_asking_for_download_link),
                ),
            ),
        )
        .branch(
            dptree::filter(|msg: Message| msg.text().map(|text| text == PROFILE_BUTTON).unwrap_or(false))
                .endpoint(handle_callback_profile_menu),
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
