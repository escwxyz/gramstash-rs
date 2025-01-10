use crate::services::cache::CacheService;
use crate::services::dialogue::DialogueState;
use crate::services::instagram::types::{InstagramIdentifier, MediaContent, PostContent};
use crate::services::ratelimiter::RateLimiter;
use crate::state::AppState;
use crate::utils::error::HandlerResult;
use crate::utils::{extract_instagram_url, keyboard, parse_url};
use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::prelude::*;
use teloxide::types::{MessageId, ParseMode};

pub async fn handle_message_asking_for_download_link(
    bot: Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: Message,
) -> HandlerResult<()> {
    info!("handle_message_asking_for_download_link");

    bot.send_message(msg.chat.id, "🔍 Please send me a message containing an Instagram content URL (post, story, reel, highlight) you want to download.")
        .parse_mode(ParseMode::Html)
        .await?;

    dialogue.update(DialogueState::AwaitingDownloadLink(msg.id)).await?;

    Ok(())
}

pub(super) async fn handle_message_awaiting_download_link(
    bot: Bot,
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
                .send_message(msg.chat.id, "❌ Please provide a valid Instagram URL.")
                .parse_mode(ParseMode::Html)
                .await?;

            dialogue.update(DialogueState::AwaitingDownloadLink(msg.id)).await?;

            return Ok(());
        }
    };

    let processing_msg = bot
        .send_message(msg.chat.id, "⏳ Processing your request...")
        .parse_mode(ParseMode::Html)
        .await?;

    let instagram_service = AppState::get()?.instagram.lock().await;
    let parsed_url = parse_url(&url)?;
    let identifier = instagram_service.parse_instagram_url(&parsed_url)?;

    let (shortcode, content_type) = match identifier {
        InstagramIdentifier::Story { username, shortcode } => (shortcode, "story"),
        InstagramIdentifier::Post { shortcode } => (shortcode, "post"),
        InstagramIdentifier::Reel { shortcode } => (shortcode, "reel"),
    };

    // check cache first

    let cached_media_info = CacheService::get_media_info(&shortcode).await?;

    info!("Checking rate limit for shortcode: {}", shortcode);

    // check rate limit
    let rate_limiter = RateLimiter::new()?;

    if !rate_limiter.check_rate_limit(msg.chat.id, &shortcode).await? {
        bot.edit_message_text(
            msg.chat.id,
            processing_msg.id,
            "⚠️ Daily download limit reached. Try again tomorrow!",
        )
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard::MainMenu::get_inline_keyboard())
        .await?;
        dialogue.update(DialogueState::Start).await?;
        return Ok(());
    }

    if let Some(media_info) = cached_media_info {
        info!("Cache hit for shortcode: {}", shortcode);
        process_media_content(&bot, &dialogue, &msg, &processing_msg, &media_info.content).await?;
        return Ok(());
    }

    let media_info = match content_type {
        "story" => {
            info!("Fetching story info for shortcode: {}", shortcode);
            todo!()
        }
        _ => instagram_service.fetch_post_info(&shortcode).await,
    };

    match media_info {
        Ok(media_info) => {
            CacheService::set_media_info(&shortcode, &media_info).await?;

            process_media_content(&bot, &dialogue, &msg, &processing_msg, &media_info.content).await?;
        }
        Err(e) => {
            let msg = bot
                .edit_message_text(
                    msg.chat.id,
                    processing_msg.id,
                    &format!("❌ Error: {}\n\nPlease try again.", e),
                )
                .await?;

            dialogue.update(DialogueState::AwaitingDownloadLink(msg.id)).await?;
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
            todo!();
        }
    };

    bot.edit_message_text(msg.chat.id, processing_msg.id, preview_text)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard::DownloadMenu::get_confirm_download_keyboard())
        .await?;

    Ok(())
}

async fn process_media_content(
    bot: &Bot,
    dialogue: &Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: &Message,
    processing_msg: &Message,
    content: &MediaContent,
) -> HandlerResult<()> {
    let content_clone = content.clone();

    dialogue
        .update(DialogueState::ConfirmDownload { content: content_clone })
        .await?;

    show_media_preview(bot, msg, processing_msg, &content).await?;

    Ok(())
}

fn validate_message(msg: &Message) -> Option<String> {
    msg.text()
        .and_then(extract_instagram_url)
        .and_then(|url| parse_url(&url).ok())
        .map(|url| url.to_string())
}
