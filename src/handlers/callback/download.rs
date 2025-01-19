use teloxide::{
    adaptors::Throttle,
    dispatching::dialogue::ErasedStorage,
    payloads::{EditMessageTextSetters, SendMessageSetters},
    prelude::{Dialogue, Requester},
    types::{InputFile, InputMedia, InputMediaPhoto, InputMediaVideo},
    Bot,
};

use teloxide::types::MaybeInaccessibleMessage;

use crate::{
    error::HandlerResult,
    handlers::RequestContext,
    services::{cache::CacheService, dialogue::DialogueState},
    utils::keyboard,
};

pub(super) async fn handle_callback_asking_for_download_link(
    bot: &Throttle<Bot>,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    message: MaybeInaccessibleMessage,
) -> HandlerResult<()> {
    info!("handle_callback_asking_for_download_link");
    bot.edit_message_text(
        message.chat().id,
        message.id(),
        t!("callbacks.download.ask_for_download_link"),
    )
    .reply_markup(keyboard::MainMenu::get_back_to_main_menu_keyboard())
    .await?;

    dialogue
        .update(DialogueState::AwaitingDownloadLink(message.id()))
        .await?;

    Ok(())
}

pub(super) async fn handle_callback_confirm_download(
    bot: &Throttle<Bot>,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    message: MaybeInaccessibleMessage,
    ctx: RequestContext,
) -> HandlerResult<()> {
    info!("handle_callback_confirm_download");

    if let Some(DialogueState::ConfirmDownload {
        shortcode,
        instagram_media,
    }) = dialogue.get().await?
    {
        bot.delete_message(message.chat().id, message.id()).await?;
        let download_msg = bot
            .send_message(message.chat().id, t!("callbacks.download.downloading"))
            .await?;

        bot.delete_message(message.chat().id, download_msg.id).await?;

        CacheService::cache_media_to_redis(
            &ctx.telegram_user_id.to_string(),
            &instagram_media.author.username,
            &shortcode,
            &instagram_media,
        )
        .await?;

        match &instagram_media.content {
            crate::services::instagram::InstagramContent::Single(media_item) => match media_item.media_type {
                crate::services::instagram::MediaType::Image => {
                    bot.send_photo(message.chat().id, InputFile::url(media_item.url.clone()))
                        .await?;
                }
                crate::services::instagram::MediaType::Video => {
                    bot.send_video(message.chat().id, InputFile::url(media_item.url.clone()))
                        .await?;
                }
            },
            crate::services::instagram::InstagramContent::Multiple(items) => {
                let media_group = items
                    .into_iter()
                    .map(|item| match item.media_type {
                        crate::services::instagram::MediaType::Image => {
                            InputMedia::Photo(InputMediaPhoto::new(InputFile::url(item.url.clone())))
                        }
                        crate::services::instagram::MediaType::Video => {
                            InputMedia::Video(InputMediaVideo::new(InputFile::url(item.url.clone())))
                        }
                    })
                    .collect::<Vec<_>>();

                bot.send_media_group(message.chat().id, media_group).await?;
            }
            crate::services::instagram::InstagramContent::Story(media_item) => match media_item.media_type {
                crate::services::instagram::MediaType::Image => {
                    bot.send_photo(message.chat().id, InputFile::url(media_item.url.clone()))
                        .await?;
                }
                crate::services::instagram::MediaType::Video => {
                    bot.send_video(message.chat().id, InputFile::url(media_item.url.clone()))
                        .await?;
                }
            },
        }
    }

    bot.send_message(message.chat().id, t!("callbacks.download.download_completed"))
        .reply_markup(keyboard::DownloadMenu::get_download_menu_inline_keyboard())
        .await?;

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
    .reply_markup(keyboard::MainMenu::get_inline_keyboard())
    .await?;

    Ok(())
}
