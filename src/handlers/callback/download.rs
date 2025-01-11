use teloxide::{
    dispatching::dialogue::ErasedStorage,
    payloads::{EditMessageTextSetters, SendMessageSetters},
    prelude::{Dialogue, Requester},
    types::{InputFile, InputMedia, InputMediaPhoto, InputMediaVideo, ParseMode},
    Bot,
};

use teloxide::types::MaybeInaccessibleMessage;

use crate::{
    services::{
        dialogue::DialogueState,
        instagram::types::{ContentType, MediaContent, PostContent},
    },
    utils::{error::HandlerResult, keyboard},
};

pub(super) async fn handle_callback_asking_for_download_link(
    bot: &Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    message: MaybeInaccessibleMessage,
) -> HandlerResult<()> {
    info!("handle_callback_asking_for_download_link");
    bot.edit_message_text(
        message.chat().id,
        message.id(),
        t!("callbacks.download.ask_for_download_link"),
    )
    .parse_mode(ParseMode::Html)
    .await?;

    dialogue
        .update(DialogueState::AwaitingDownloadLink(message.id()))
        .await?;

    Ok(())
}

pub(super) async fn handle_callback_confirm_download(
    bot: &Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    message: MaybeInaccessibleMessage,
) -> HandlerResult<()> {
    info!("handle_callback_confirm_download");
    if let Some(DialogueState::ConfirmDownload { content }) = dialogue.get().await? {
        bot.delete_message(message.chat().id, message.id()).await?;
        let download_msg = bot
            .send_message(message.chat().id, t!("callbacks.download.downloading"))
            .parse_mode(ParseMode::Html)
            .await?;

        bot.delete_message(message.chat().id, download_msg.id).await?;

        match content {
            MediaContent::Post(PostContent::Single { url, content_type }) => {
                let _ = match content_type {
                    // TODO add caption and meta data etc
                    ContentType::Image => bot.send_photo(message.chat().id, InputFile::url(url)).await?,
                    ContentType::Reel => bot.send_video(message.chat().id, InputFile::url(url)).await?,
                };
            }
            MediaContent::Post(PostContent::Carousel { items }) => {
                let media_group = items
                    .into_iter()
                    .map(|item| match item.content_type {
                        ContentType::Image => InputMedia::Photo(InputMediaPhoto::new(InputFile::url(item.url))),
                        ContentType::Reel => InputMedia::Video(InputMediaVideo::new(InputFile::url(item.url))),
                    })
                    .collect::<Vec<_>>();

                bot.send_media_group(message.chat().id, media_group).await?;
            }
            MediaContent::Story(_story_content) => todo!(),
        }
    }

    bot.send_message(message.chat().id, t!("callbacks.download.download_completed"))
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard::DownloadMenu::get_download_menu_inline_keyboard())
        .await?;

    Ok(())
}

pub(super) async fn handle_callback_cancel_download(bot: &Bot, message: MaybeInaccessibleMessage) -> HandlerResult<()> {
    info!("handle_callback_cancel_download");
    bot.edit_message_text(
        message.chat().id,
        message.id(),
        t!("callbacks.download.cancel_download"),
    )
    .reply_markup(keyboard::MainMenu::get_inline_keyboard())
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}
