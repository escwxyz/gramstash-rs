use teloxide::{
    adaptors::Throttle,
    dispatching::dialogue::ErasedStorage,
    payloads::EditMessageTextSetters,
    prelude::{Dialogue, Requester},
    Bot,
};

use teloxide::types::MaybeInaccessibleMessage;

use crate::{
    error::HandlerResult,
    handler::keyboard::{get_back_to_main_menu_keyboard, get_main_menu_keyboard, get_platform_keyboard},
    platform::{Platform, PostDownloadState},
    service::dialogue::model::DialogueState,
    state::AppState,
};

pub(super) async fn handle_callback_select_platform(
    bot: &Throttle<Bot>,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    message: MaybeInaccessibleMessage,
) -> HandlerResult<()> {
    info!("handle_callback_select_platform");

    bot.edit_message_text(
        message.chat().id,
        message.id(),
        t!("callbacks.download.select_platform"),
    )
    .reply_markup(get_platform_keyboard().await?)
    .await?;

    dialogue.update(DialogueState::SelectPlatform).await?;

    Ok(())
}

pub(super) async fn handle_callback_asking_for_download_link(
    bot: &Throttle<Bot>,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    message: MaybeInaccessibleMessage,
    platform: Platform,
) -> HandlerResult<()> {
    info!("handle_callback_asking_for_download_link");
    bot.edit_message_text(
        message.chat().id,
        message.id(),
        t!(
            "callbacks.download.ask_for_download_link",
            platform = platform.to_string()
        ),
    )
    .reply_markup(get_back_to_main_menu_keyboard())
    .await?;

    dialogue
        .update(DialogueState::AwaitingDownloadLink {
            message_id: message.id(),
            platform,
        })
        .await?;

    Ok(())
}

pub(super) async fn handle_callback_confirm_download(
    bot: &Throttle<Bot>,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    message: MaybeInaccessibleMessage,
) -> HandlerResult<()> {
    info!("handle_callback_confirm_download");

    if let Some(DialogueState::ConfirmDownload { media_info }) = dialogue.get().await? {
        let queue_manager = &AppState::get()?.runtime.queue_manager;

        let state = queue_manager
            .handle_download_confirmation(&media_info.identifier)
            .await?;

        match state {
            PostDownloadState::Success => dialogue.update(DialogueState::Start).await?,

            PostDownloadState::Error => {
                bot.edit_message_text(message.chat().id, message.id(), t!("callbacks.download.error"))
                    .await?;

                // TODO: 重试
            }
        }
    }

    Ok(())
}

pub(super) async fn handle_callback_cancel_download(
    bot: &Throttle<Bot>,
    message: MaybeInaccessibleMessage,
) -> HandlerResult<()> {
    info!("handle_callback_cancel_download");
    bot.edit_message_text(
        message.chat().id,
        message.id(),
        t!("callbacks.download.cancel_download"),
    )
    .reply_markup(get_main_menu_keyboard())
    .await?;

    Ok(())
}
