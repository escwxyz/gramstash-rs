use serde::{Deserialize, Serialize};
use teloxide::types::MessageId;

use crate::platform::{MediaFile, Platform};

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub enum DialogueState {
    #[default]
    Start,
    // Download
    SelectPlatform,
    AwaitingDownloadLink {
        message_id: MessageId,
        platform: Platform,
    },
    ConfirmDownload {
        media_file: MediaFile,
    },
    // Authentication
    AwaitingUsername(MessageId),
    AwaitingPassword {
        username: String,
        prompt_msg_id: MessageId,
    },
    ConfirmLogout,
}
