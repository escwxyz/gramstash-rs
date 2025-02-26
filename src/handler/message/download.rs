use crate::context::UserContext;
use crate::error::{BotError, HandlerResult};

use crate::handler::keyboard::get_back_to_main_menu_keyboard;

use crate::platform::{extract_url_from_message, Platform};
use crate::runtime::{DownloadTask, TaskContext};
use crate::service::dialogue::model::DialogueState;

use crate::state::AppState;
use teloxide::{adaptors::Throttle, dispatching::dialogue::ErasedStorage, prelude::*, types::MessageId};

pub(super) async fn handle_message_awaiting_download_link(
    bot: Throttle<Bot>,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: Message,
    (message_id, platform): (MessageId, Platform),
) -> HandlerResult<()> {
    info!("handle_message_awaiting_download_link");

    bot.delete_message(msg.chat.id, message_id).await?;

    let processing_msg = bot
        .send_message(msg.chat.id, t!("messages.download.processing_request"))
        .await?;

    let url_str = match msg.text().and_then(|text| extract_url_from_message(&platform, text)) {
        Some(url) => url,
        None => {
            let msg = bot
                .send_message(msg.chat.id, t!("messages.download.invalid_url"))
                .reply_markup(get_back_to_main_menu_keyboard())
                .await?;

            dialogue
                .update(DialogueState::AwaitingDownloadLink {
                    message_id: msg.id,
                    platform,
                })
                .await
                .map_err(|e| BotError::DialogueStateError(e.to_string()))?;

            return Ok(());
        }
    };

    bot.delete_message(msg.chat.id, msg.id).await?; // Delete the URL message from User

    let context = UserContext::global();

    let download_task = DownloadTask::new(
        url_str,
        TaskContext {
            user_id: context.user_id().0,
            chat_id: msg.chat.id.0,
            message_id: processing_msg.id.0,
            user_tier: context.user_tier().await,
            platform,
        },
    );

    let queue_manager = &AppState::get()?.runtime.queue_manager;

    let state = queue_manager.push_download_task(download_task).await?;

    match state {
        crate::platform::DownloadState::RateLimited => {
            dialogue.update(DialogueState::Start).await?;
        }
        crate::platform::DownloadState::Success(media_file) => {
            dialogue.update(DialogueState::ConfirmDownload { media_file }).await?;
        }
        crate::platform::DownloadState::Error => {
            dialogue.update(DialogueState::Start).await?;
        }
    }

    Ok(())
}
