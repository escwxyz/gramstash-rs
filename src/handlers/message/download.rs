use crate::error::{BotError, BotResult, HandlerResult};

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

    let telegram_user_id = ctx.telegram_user_id.to_string();

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

    let (identifier, content_type, target_instagram_username) = {
        let identifier = AppState::get()?.instagram.parse_instagram_url(&parsed_url)?;

        match identifier {
            InstagramIdentifier::Story { username, story_id } => (story_id, "story", Some(username)),
            InstagramIdentifier::Post { shortcode } => (shortcode, "post", None),
            InstagramIdentifier::Reel { shortcode } => (shortcode, "reel", None),
        }
    };

    // check cache first

    let cached_instagram_media = CacheService::get_media_from_redis(&telegram_user_id, &identifier).await?;

    info!("Checking rate limit for: {}", identifier);

    // check rate limit
    let rate_limiter = RateLimiter::new()?;

    if !rate_limiter.check_rate_limit(&telegram_user_id, &identifier).await? {
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

    if let Some(instagram_media) = cached_instagram_media {
        info!("Cache hit for identifier: {}", identifier);
        process_media_content(&bot, &dialogue, &msg, &processing_msg.id, &identifier, &instagram_media).await?;
        return Ok(());
    }

    let state = AppState::get()?;

    let (instagram_media, message_to_edit): (BotResult<InstagramMedia>, MessageId) = match content_type {
        "story" => {
            let validating_msg = bot
                .edit_message_text(
                    msg.chat.id,
                    processing_msg.id,
                    t!("messages.download.download_story.validating_session"),
                )
                .await?;

            let session_data = state.session.get_valid_session(&msg.chat.id.to_string()).await?;

            if let Some(session_data) = session_data {
                let mut auth_service = state.auth.lock().await;
                auth_service.restore_cookies(&session_data)?;
                let http = HttpService::new(false, DeviceType::Desktop, Some(auth_service.client.clone()))?;

                let fetching_stories_msg = bot
                    .edit_message_text(
                        msg.chat.id,
                        validating_msg.id,
                        t!("messages.download.download_story.fetching_stories"),
                    )
                    .await?;

                (
                    state
                        .instagram
                        .get_story(
                            &telegram_user_id,
                            &target_instagram_username.unwrap_or_default(),
                            &identifier,
                            &http,
                        )
                        .await,
                    fetching_stories_msg.id,
                )
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
            (instagram_service.fetch_post_info(&identifier).await, processing_msg.id)
        }
    };

    match instagram_media {
        Ok(instagram_media) => {
            process_media_content(&bot, &dialogue, &msg, &message_to_edit, &identifier, &instagram_media).await?;
        }
        Err(e) => {
            let msg = bot
                .edit_message_text(
                    msg.chat.id,
                    message_to_edit,
                    t!("messages.download.download_failed", error = e.to_string()),
                )
                .reply_markup(keyboard::DownloadMenu::get_download_menu_inline_keyboard())
                .await?;

            dialogue
                .update(DialogueState::AwaitingDownloadLink(msg.id))
                .await
                .map_err(|e| BotError::DialogueStateError(e.to_string()))?;
        }
    }

    Ok(())
}

async fn show_media_preview(
    bot: &Throttle<Bot>,
    msg: &Message,
    processing_msg_id: &MessageId,
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
                "messages.download.media_preview_signle_story",
                username = instagram_media.author.username,
                timestamp = instagram_media.timestamp,
                caption = instagram_media.caption.clone().unwrap_or_default()
            )
        }
    };

    bot.edit_message_text(msg.chat.id, *processing_msg_id, preview_text)
        .reply_markup(keyboard::DownloadMenu::get_confirm_download_keyboard())
        .await?;

    Ok(())
}

async fn process_media_content(
    bot: &Throttle<Bot>,
    dialogue: &Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: &Message,
    processing_msg_id: &MessageId,
    identifier: &str,
    instagram_media: &InstagramMedia,
) -> HandlerResult<()> {
    let instagram_media_clone = instagram_media.clone();

    dialogue
        .update(DialogueState::ConfirmDownload {
            identifier: identifier.to_string(),
            instagram_media: instagram_media_clone,
        })
        .await?;

    show_media_preview(bot, msg, processing_msg_id, &instagram_media).await?;

    Ok(())
}

fn validate_message(msg: &Message) -> Option<String> {
    msg.text()
        .and_then(extract_instagram_url)
        .and_then(|url| parse_url(&url).ok())
        .map(|url| url.to_string())
}
