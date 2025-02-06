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

    bot.delete_message(msg.chat.id, message_id).await?; // TODO

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

    let download_task = DownloadTask::new(
        url_str,
        TaskContext {
            user_id: UserContext::global().user_id().0,
            chat_id: msg.chat.id.0,
            message_id: processing_msg.id.0,
            user_tier: UserContext::global().user_tier().await,
            platform,
        },
    );

    let queue_manager = &AppState::get()?.runtime.queue_manager;

    let state = queue_manager.push_download_task(download_task).await?;

    match state {
        crate::platform::DownloadState::RateLimited => {
            dialogue.update(DialogueState::Start).await?;
        }
        crate::platform::DownloadState::Success(media_info) => {
            dialogue.update(DialogueState::ConfirmDownload { media_info }).await?;
        }
        crate::platform::DownloadState::Error => {
            dialogue.update(DialogueState::Start).await?;
        }
    }

    // let (instagram_media, message_to_edit): (BotResult<InstagramMedia>, MessageId) = match content_type {
    //     "story" => {
    //         todo!()
    //         // let validating_msg = bot
    //         //     .edit_message_text(
    //         //         msg.chat.id,
    //         //         processing_msg.id,
    //         //         t!("messages.download.download_story.validating_session"),
    //         //     )
    //         //     .await?;

    //         // let session_data = state.session.get_valid_session(&msg.chat.id.to_string()).await?;

    //         // if let Some(session_data) = session_data {
    //         //     let mut auth_service = state.auth.lock().await;
    //         //     auth_service.restore_cookies(&session_data)?;
    //         //     let http = HttpService::new(false, DeviceType::Desktop, Some(auth_service.client.clone()))?;

    //         //     let fetching_stories_msg = bot
    //         //         .edit_message_text(
    //         //             msg.chat.id,
    //         //             validating_msg.id,
    //         //             t!("messages.download.download_story.fetching_stories"),
    //         //         )
    //         //         .await?;

    //         //     (
    //         //         state
    //         //             .instagram
    //         //             .get_story(
    //         //                 &telegram_user_id,
    //         //                 &target_instagram_username.unwrap_or_default(),
    //         //                 &identifier,
    //         //                 &http,
    //         //             )
    //         //             .await,
    //         //         fetching_stories_msg.id,
    //         //     )
    //         // } else {
    //         //     bot.edit_message_text(msg.chat.id, processing_msg.id, t!("messages.download.session_expired"))
    //         //         .reply_markup(keyboard::ProfileMenu::get_profile_menu_inline_keyboard(false))
    //         //         .await?;

    //         //     dialogue
    //         //         .update(DialogueState::Start)
    //         //         .await
    //         //         .map_err(|e| BotError::DialogueStateError(e.to_string()))?;

    //         //     return Ok(());
    //         // }
    //     }
    //     _ => (instagram_service.fetch_resource(&identifier).await, processing_msg.id),
    // };

    Ok(())
}
