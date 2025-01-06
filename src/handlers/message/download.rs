use crate::services::cache::CacheService;
use crate::services::dialogue::DialogueState;
use crate::services::instagram::types::{ContentType, InstagramIdentifier, MediaContent, PostContent};
use crate::services::instagram::InstagramService;
use crate::services::ratelimiter::RateLimiter;
use crate::utils::error::{BotError, HandlerResult};
use crate::utils::{extract_instagram_url, keyboard, parse_url};
use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::prelude::*;
use teloxide::types::{InputFile, InputMedia, InputMediaPhoto, InputMediaVideo, MaybeInaccessibleMessage, MessageId};

pub async fn handle_download(bot: Bot, message: MaybeInaccessibleMessage, content: MediaContent) -> HandlerResult<()> {
    bot.delete_message(message.chat().id, message.id()).await?;

    let download_msg = bot.send_message(message.chat().id, "⬇️ Downloading...").await?;

    match content {
        MediaContent::Post(post_content) => {
            info!("Downloading post: {:?}", post_content);
            match post_content {
                PostContent::Single { url, content_type } => {
                    bot.delete_message(message.chat().id, download_msg.id).await?;
                    let _ = match content_type {
                        ContentType::Image => bot.send_photo(message.chat().id, InputFile::url(url)).await?,
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
                            ContentType::Image => InputMedia::Photo(InputMediaPhoto::new(InputFile::url(item.url))),
                            ContentType::Reel => InputMedia::Video(InputMediaVideo::new(InputFile::url(item.url))),
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
    }

    Ok(())
}

pub async fn handle_post_link(
    bot: Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: Message,
    message_id: MessageId,
) -> HandlerResult<()> {
    bot.delete_message(msg.chat.id, message_id).await?;

    let url = match validate_message(&msg) {
        Some(url) => url,
        None => {
            let msg = bot
                .send_message(msg.chat.id, "❌ Please provide a valid Instagram URL.")
                .await?;

            dialogue
                .update(DialogueState::AwaitingPostLink(msg.id))
                .await
                .map_err(|e| BotError::DialogueError(e.to_string()))?;

            return Ok(());
        }
    };

    let processing_msg = bot.send_message(msg.chat.id, "⏳ Processing your request...").await?;

    let instagram_service = InstagramService::new();
    let parsed_url = parse_url(&url)?;
    let identifier = instagram_service.parse_instagram_url(&parsed_url)?;
    let shortcode = match identifier {
        InstagramIdentifier::Post { shortcode } => shortcode,
        InstagramIdentifier::Story { shortcode, .. } => shortcode,
        InstagramIdentifier::Reel { shortcode, .. } => shortcode,
    };

    // check cache first

    let cached_media_info = CacheService::get_media_info(&shortcode).await?;

    info!("Checking rate limit for shortcode: {}", shortcode);

    // check rate limit
    let rate_limiter = RateLimiter::new();

    if !rate_limiter.check_rate_limit(msg.chat.id, &shortcode).await? {
        bot.edit_message_text(
            msg.chat.id,
            processing_msg.id,
            "⚠️ Daily download limit reached. Try again tomorrow!",
        )
        .reply_markup(keyboard::get_back_to_menu_keyboard())
        .await?;

        dialogue
            .exit()
            .await
            .map_err(|e| BotError::DialogueError(e.to_string()))?;

        return Ok(());
    }

    if let Some(media_info) = cached_media_info {
        info!("Cache hit for shortcode: {}", shortcode);
        process_media_content(&bot, &dialogue, &msg, &processing_msg, media_info.content).await?;
        return Ok(());
    }

    match instagram_service.fetch_post_info(&shortcode).await {
        Ok(media_info) => {
            CacheService::set_media_info(&shortcode, &media_info).await?;

            process_media_content(&bot, &dialogue, &msg, &processing_msg, media_info.content).await?;
        }
        Err(e) => {
            handle_error(&bot, &msg, e, &processing_msg).await?;
            dialogue
                .exit()
                .await
                .map_err(|e| BotError::DialogueError(e.to_string()))?;
        }
    }

    Ok(())
}

// TODO: implement media preview with better UI and more information
async fn show_media_preview(
    bot: &Bot,
    msg: &Message,
    processing_msg: &Message,
    content: &MediaContent,
) -> ResponseResult<()> {
    let preview_text = match content {
        MediaContent::Post(post_content) => match post_content {
            PostContent::Single { content_type, .. } => {
                format!("Found a single {:?} post. Would you like to download it?", content_type)
            }
            PostContent::Carousel { items, .. } => {
                format!(
                    "Found a carousel with {} items. Would you like to download it?",
                    items.len()
                )
            }
        },
        MediaContent::Story(_) => {
            handle_error(
                bot,
                msg,
                BotError::InstagramApi("this link is a story, not a post".to_string()),
                processing_msg,
            )
            .await?;
            return Ok(());
        }
    };

    bot.edit_message_text(msg.chat.id, processing_msg.id, preview_text)
        .reply_markup(keyboard::get_confirm_download_keyboard())
        .await?;

    Ok(())
}

async fn process_media_content(
    bot: &Bot,
    dialogue: &Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: &Message,
    processing_msg: &Message,
    content: MediaContent,
) -> ResponseResult<()> {
    let content_clone = content.clone();

    dialogue
        .update(DialogueState::ConfirmDownload { content: content_clone })
        .await
        .map_err(|e| BotError::DialogueError(e.to_string()))?;

    show_media_preview(bot, msg, processing_msg, &content).await?;

    Ok(())
}

async fn handle_error(bot: &Bot, msg: &Message, error: BotError, processing_msg: &Message) -> ResponseResult<()> {
    info!("Error: {:?}", error);
    bot.edit_message_text(msg.chat.id, processing_msg.id, &format!("❌ Error: {}", error))
        .await?;
    Ok(())
}

// TODO: implement story link handler
pub async fn handle_story_link(
    bot: Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: Message,
) -> HandlerResult<()> {
    info!("Story link handler");
    bot.send_message(msg.chat.id, "Story downloading is in progress...")
        .await?;
    dialogue
        .reset()
        .await
        .map_err(|e| BotError::DialogueError(e.to_string()))?;
    Ok(())
}

fn validate_message(msg: &Message) -> Option<String> {
    msg.text()
        .and_then(extract_instagram_url)
        .and_then(|url| parse_url(&url).ok())
        .map(|url| url.to_string())
}

async fn send_download_complete_message(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    bot.send_message(chat_id, "✅ Download completed! What would you like to do next?")
        .reply_markup(keyboard::get_back_to_menu_keyboard())
        .await?;

    Ok(())
}
