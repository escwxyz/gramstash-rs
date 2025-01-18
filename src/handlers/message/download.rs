use crate::error::{BotError, HandlerResult};

use crate::services::cache::CacheService;
use crate::services::dialogue::DialogueState;
use crate::services::instagram::{InstagramIdentifier, MediaContent, MediaInfo, PostContent};

use crate::services::ratelimiter::RateLimiter;
use crate::state::AppState;
use crate::utils::{extract_instagram_url, keyboard, parse_url};
use teloxide::adaptors::Throttle;
use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::prelude::*;
use teloxide::types::MessageId;

pub(super) async fn handle_message_awaiting_download_link(
    bot: Throttle<Bot>,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: Message,
    message_id: MessageId,
) -> HandlerResult<()> {
    info!("handle_message_awaiting_download_link");

    bot.delete_message(msg.chat.id, message_id).await?; // TODO

    let url = match validate_message(&msg) {
        Some(url) => url,
        None => {
            let msg = bot
                .send_message(msg.chat.id, t!("messages.download.invalid_url"))
                .reply_markup(keyboard::MainMenu::get_back_to_main_menu_keyboard())
                .await?;

            dialogue
                .update(DialogueState::AwaitingDownloadLink(msg.id))
                .await
                .map_err(|e| BotError::DialogueStateError(e.to_string()))?;

            return Ok(());
        }
    };

    let processing_msg = bot
        .send_message(msg.chat.id, t!("messages.download.processing_request"))
        .await?;

    let parsed_url = parse_url(&url)?;

    let (shortcode, content_type) = {
        let identifier = AppState::get()?.instagram.parse_instagram_url(&parsed_url)?;

        match identifier {
            InstagramIdentifier::Story { username: _, shortcode } => (shortcode, "story"),
            InstagramIdentifier::Post { shortcode } => (shortcode, "post"),
            InstagramIdentifier::Reel { shortcode } => (shortcode, "reel"),
        }
    };

    // check cache first

    let cached_media_info = CacheService::get_media_info(&shortcode).await?;

    info!("Checking rate limit for shortcode: {}", shortcode);

    // check rate limit
    let rate_limiter = RateLimiter::new()?;

    if !rate_limiter
        // In private chat, msg.chat.id is the same as UserId
        .check_rate_limit(&msg.chat.id.to_string(), &shortcode)
        .await?
    {
        bot.edit_message_text(
            msg.chat.id,
            processing_msg.id,
            t!("messages.download.download_limit_reached"),
        )
        .reply_markup(keyboard::MainMenu::get_inline_keyboard())
        .await?;
        dialogue
            .update(DialogueState::Start)
            .await
            .map_err(|e| BotError::DialogueStateError(e.to_string()))?;
        return Ok(());
    }

    if let Some(media_info) = cached_media_info {
        info!("Cache hit for shortcode: {}", shortcode);
        process_media_content(&bot, &dialogue, &msg, &processing_msg, &shortcode, &media_info).await?;
        return Ok(());
    }

    let instagram_service = AppState::get()?.instagram.clone();

    let media_info = match content_type {
        "story" => {
            todo!()
        }
        _ => instagram_service.fetch_post_info(&shortcode).await,
    };

    match media_info {
        Ok(media_info) => {
            process_media_content(&bot, &dialogue, &msg, &processing_msg, &shortcode, &media_info).await?;
        }
        Err(e) => {
            let msg = bot
                .edit_message_text(
                    msg.chat.id,
                    processing_msg.id,
                    &format!("❌ Error: {}\n\nPlease try again.", e), // TODO: translate & reply_markup
                )
                .await?;

            dialogue
                .update(DialogueState::AwaitingDownloadLink(msg.id))
                .await
                .map_err(|e| BotError::DialogueStateError(e.to_string()))?;
        }
    }

    Ok(())
}

// TODO: implement media preview with better UI and more information
async fn show_media_preview(
    bot: &Throttle<Bot>,
    msg: &Message,
    processing_msg: &Message,
    media_info: &MediaInfo,
) -> ResponseResult<()> {
    let preview_text = match &media_info.content {
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
            todo!();
        }
    };

    bot.edit_message_text(msg.chat.id, processing_msg.id, preview_text)
        .reply_markup(keyboard::DownloadMenu::get_confirm_download_keyboard())
        .await?;

    Ok(())
}

async fn process_media_content(
    bot: &Throttle<Bot>,
    dialogue: &Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: &Message,
    processing_msg: &Message,
    shortcode: &str,
    media_info: &MediaInfo,
) -> HandlerResult<()> {
    let media_info_clone = media_info.clone();

    dialogue
        .update(DialogueState::ConfirmDownload {
            shortcode: shortcode.to_string(),
            media_info: media_info_clone,
        })
        .await?;

    show_media_preview(bot, msg, processing_msg, &media_info).await?;

    Ok(())
}

fn validate_message(msg: &Message) -> Option<String> {
    msg.text()
        .and_then(extract_instagram_url)
        .and_then(|url| parse_url(&url).ok())
        .map(|url| url.to_string())
}
