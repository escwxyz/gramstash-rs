use crate::{
    bot::DialogueState,
    services::instagram::types::{ContentType, MediaContent, PostContent},
    utils::{error::BotError, keyboard},
};
use teloxide::{
    dispatching::dialogue::ErasedStorage,
    prelude::*,
    types::{CallbackQuery, InputFile, InputMedia, InputMediaPhoto, InputMediaVideo, ParseMode},
};

pub async fn handle_callback(
    bot: Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    q: CallbackQuery,
) -> ResponseResult<()> {
    let data = q
        .data
        .ok_or_else(|| BotError::InvalidState("No callback data".into()))?;

    let message = q.message.ok_or_else(|| BotError::InvalidState("No message".into()))?;

    match data.as_str() {
        "download_menu" => {
            bot.edit_message_text(message.chat().id, message.id(), "What would you like to download?")
                .reply_markup(keyboard::get_download_menu_keyboard())
                .await
                .map_err(|e| BotError::Other(e.into()))?;
        }
        "download_post" => {
            let msg = bot
                .edit_message_text(
                    message.chat().id,
                    message.id(),
                    "Please send me the Instagram post or reel URL you want to download.",
                )
                .await
                .map_err(|e| BotError::Other(e.into()))?;

            dialogue
                .update(DialogueState::AwaitingPostLink { message_id: msg.id })
                .await
                .map_err(|e| BotError::DialogueError(e.to_string()))?;
        }
        "download_story" => {
            let msg = bot
                .edit_message_text(
                    message.chat().id,
                    message.id(),
                    "Please send me the Instagram story URL you want to download.",
                )
                .await
                .map_err(|e| BotError::Other(e.into()))?;

            dialogue
                .update(DialogueState::AwaitingStoryLink { message_id: msg.id })
                .await
                .map_err(|e| BotError::DialogueError(e.to_string()))?;
        }

        "confirm" => {
            if let Some(DialogueState::ConfirmDownload { content }) = dialogue
                .get()
                .await
                .map_err(|e| BotError::DialogueError(e.to_string()))?
            {
                bot.delete_message(message.chat().id, message.id()).await?;

                let download_msg = bot.send_message(message.chat().id, "⬇️ Downloading...").await?;

                match content {
                    MediaContent::Post(post_content) => {
                        info!("Downloading post: {:?}", post_content);
                        match post_content {
                            PostContent::Single { url, content_type } => {
                                bot.delete_message(message.chat().id, download_msg.id).await?;
                                let _ = match content_type {
                                    ContentType::Image => {
                                        bot.send_photo(message.chat().id, InputFile::url(url)).await?
                                    }
                                    ContentType::Reel => bot.send_video(message.chat().id, InputFile::url(url)).await?,
                                };

                                send_download_complete_message(&bot, message.chat().id).await?;
                            }
                            PostContent::Carousel { items } => {
                                // Delete the downloading message
                                bot.delete_message(message.chat().id, download_msg.id).await?;
                                let media_group = items
                                    .into_iter()
                                    .map(|item| match item.content_type {
                                        ContentType::Image => {
                                            InputMedia::Photo(InputMediaPhoto::new(InputFile::url(item.url)))
                                        }
                                        ContentType::Reel => {
                                            InputMedia::Video(InputMediaVideo::new(InputFile::url(item.url)))
                                        }
                                    })
                                    .collect::<Vec<_>>();

                                bot.send_media_group(message.chat().id, media_group).await?;

                                send_download_complete_message(&bot, message.chat().id).await?;
                            }
                        }
                    }
                    // TODO: implement story download
                    MediaContent::Story(story_content) => {
                        info!("Downloading story: {:?}", story_content);
                        bot.delete_message(message.chat().id, download_msg.id).await?;
                        bot.send_message(message.chat().id, "Story downloading is in progress...")
                            .await?;
                    }
                };
                dialogue
                    .update(DialogueState::Start)
                    .await
                    .map_err(|e| BotError::DialogueError(e.to_string()))?;
            }
        }

        "cancel" => {
            dialogue
                .update(DialogueState::Start)
                .await
                .map_err(|e| BotError::DialogueError(e.to_string()))?;

            bot.edit_message_text(
                message.chat().id,
                message.id(),
                "Download cancelled. What would you like to do?",
            )
            .reply_markup(keyboard::get_main_menu_keyboard())
            .await?;
        }

        // ------------
        "settings_menu" => {
            info!("Showing settings menu");
            bot.edit_message_text(message.chat().id, message.id(), "⚙️ Settings")
                .reply_markup(keyboard::get_settings_keyboard())
                .await
                .map_err(|e| BotError::Other(e.into()))?;
        }
        "help_menu" => {
            info!("Showing help menu");

            bot.edit_message_text(
                message.chat().id,
                message.id(),
                "ℹ️ Help and Information\n\n\
                 /start \\- Start the bot\n\
                 /help \\- Show this help message",
            )
            .parse_mode(ParseMode::MarkdownV2)
            .await
            .map_err(|e| BotError::Other(e.into()))?;
        }
        "main_menu" => {
            info!("Exiting dialogue");

            dialogue
                .exit()
                .await
                .map_err(|e| BotError::DialogueError(e.to_string()))?;

            bot.edit_message_text(message.chat().id, message.id(), "Please select an option:")
                .reply_markup(keyboard::get_main_menu_keyboard())
                .await
                .map_err(|e| BotError::Other(e.into()))?;
        }
        _ => {
            bot.answer_callback_query(&q.id)
                .text("Unknown command")
                .await
                .map_err(|e| BotError::Other(e.into()))?;
        }
    }

    bot.answer_callback_query(&q.id)
        .await
        .map_err(|e| BotError::Other(e.into()))?;

    Ok(())
}

async fn send_download_complete_message(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    bot.send_message(chat_id, "✅ Download completed! What would you like to do next?")
        .reply_markup(keyboard::get_back_to_menu_keyboard())
        .await?;

    Ok(())
}
