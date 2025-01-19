use crate::error::{BotError, HandlerResult};

use crate::handlers::RequestContext;
use crate::services::cache::CacheService;
use crate::services::dialogue::DialogueState;
use crate::services::http::{DeviceType, HttpService};
use crate::services::instagram::{InstagramIdentifier, InstagramMedia};

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
    ctx: RequestContext,
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

    let (shortcode, content_type, instagram_username) = {
        let identifier = AppState::get()?.instagram.parse_instagram_url(&parsed_url)?;

        match identifier {
            InstagramIdentifier::Story { username, shortcode } => (shortcode, "story", Some(username)),
            InstagramIdentifier::Post { shortcode } => (shortcode, "post", None),
            InstagramIdentifier::Reel { shortcode } => (shortcode, "reel", None),
        }
    };

    // check cache first

    let cached_media_info = CacheService::get_media_from_redis(&ctx.telegram_user_id.to_string(), &shortcode).await?;

    info!("Checking rate limit for shortcode: {}", shortcode);

    // check rate limit
    let rate_limiter = RateLimiter::new()?;

    if !rate_limiter
        .check_rate_limit(&ctx.telegram_user_id.to_string(), &shortcode)
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

    let state = AppState::get()?;

    let media_info = match content_type {
        "story" => {
            let session_data = state.session.get_valid_session(&msg.chat.id.to_string()).await?;

            // todo!()
            if let Some(session_data) = session_data {
                let mut auth_service = state.auth.lock().await;
                auth_service.restore_cookies(&session_data)?;
                let http = HttpService::new(false, DeviceType::Desktop, Some(auth_service.client.clone()))?;
                state
                    .instagram
                    .get_story(
                        &ctx.telegram_user_id.to_string(),
                        &instagram_username.unwrap_or_default(),
                        &shortcode,
                        &http,
                    )
                    .await
            } else {
                bot.edit_message_text(msg.chat.id, processing_msg.id, t!("messages.download.session_expired"))
                    .reply_markup(keyboard::ProfileMenu::get_profile_menu_inline_keyboard(false))
                    .await?;

                dialogue
                    .update(DialogueState::Start)
                    .await
                    .map_err(|e| BotError::DialogueStateError(e.to_string()))?;

                return Ok(());
            }
        }
        _ => {
            let instagram_service = state.instagram;
            instagram_service.fetch_post_info(&shortcode).await
        }
    };

    match media_info {
        Ok(instagram_media) => {
            process_media_content(&bot, &dialogue, &msg, &processing_msg, &shortcode, &instagram_media).await?;
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

// TODO
async fn show_media_preview(
    bot: &Throttle<Bot>,
    msg: &Message,
    processing_msg: &Message,
    instagram_media: &InstagramMedia,
) -> ResponseResult<()> {
    let preview_text = match &instagram_media.content {
        crate::services::instagram::InstagramContent::Single(media_item) => match media_item.media_type {
            crate::services::instagram::MediaType::Image => {
                t!(
                    "messages.download.media_preview_single_image",
                    username = instagram_media.author.username,
                    timestamp = instagram_media.timestamp,
                    caption = instagram_media.caption.clone().unwrap_or_default()
                )
            }
            crate::services::instagram::MediaType::Video => {
                t!(
                    "messages.download.media_preview_single_video",
                    username = instagram_media.author.username,
                    timestamp = instagram_media.timestamp,
                    caption = instagram_media.caption.clone().unwrap_or_default()
                )
            }
        },
        crate::services::instagram::InstagramContent::Multiple(items) => {
            let item = items.first().unwrap();

            match item.media_type {
                crate::services::instagram::MediaType::Image => {
                    t!(
                        "messages.download.media_preview_multiple_images",
                        count = items.len(),
                        username = instagram_media.author.username,
                        timestamp = instagram_media.timestamp,
                        caption = instagram_media.caption.clone().unwrap_or_default()
                    )
                }
                crate::services::instagram::MediaType::Video => {
                    t!(
                        "messages.download.media_preview_multiple_videos",
                        count = items.len(),
                        username = instagram_media.author.username,
                        timestamp = instagram_media.timestamp,
                        caption = instagram_media.caption.clone().unwrap_or_default()
                    )
                }
            }
        }
        crate::services::instagram::InstagramContent::Story(_) => {
            t!(
                "messages.download.media_preview_story",
                count = 1,
                username = instagram_media.author.username,
                timestamp = instagram_media.timestamp,
                caption = instagram_media.caption.clone().unwrap_or_default()
            )
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
    instagram_media: &InstagramMedia,
) -> HandlerResult<()> {
    let instagram_media_clone = instagram_media.clone();

    dialogue
        .update(DialogueState::ConfirmDownload {
            shortcode: shortcode.to_string(),
            instagram_media: instagram_media_clone,
        })
        .await?;

    show_media_preview(bot, msg, processing_msg, &instagram_media).await?;

    Ok(())
}

fn validate_message(msg: &Message) -> Option<String> {
    msg.text()
        .and_then(extract_instagram_url)
        .and_then(|url| parse_url(&url).ok())
        .map(|url| url.to_string())
}
