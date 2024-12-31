use crate::handlers::login;
use crate::services::cache::CacheService;
use crate::services::instagram::{InstagramService, MediaInfo, MediaType};
use crate::services::ratelimiter::RateLimiter;
use crate::utils::parse_url;
use anyhow::{anyhow, Result};
use teloxide::prelude::*;
use teloxide::types::InputFile;

pub async fn handle(
    bot: Bot,
    msg: Message,
    url: String,
    instagram_service: &InstagramService,
    rate_limiter: &RateLimiter,
) -> ResponseResult<()> {
    let processing_msg = bot.send_message(msg.chat.id, "⏳ Processing your request...").await?;

    info!("Validating URL...");
    if !url.contains("instagram.com") {
        bot.edit_message_text(
            msg.chat.id,
            processing_msg.id,
            "❌ Please provide a valid Instagram URL",
        )
        .await?;
        return Ok(());
    }

    info!("Processing the download...");

    match process_download(&bot, &msg, &url, instagram_service, rate_limiter).await {
        Ok(status) => {
            if status < 0 {
                info!("Reached daily download limit");

                bot.edit_message_text(
                    msg.chat.id,
                    processing_msg.id,
                    "⚠️ Daily download limit reached. Try again tomorrow!",
                )
                .await?;
            } else {
                info!("Download completed!");
                bot.edit_message_text(msg.chat.id, processing_msg.id, "✅ Download completed!")
                    .await?;
            }
        }
        Err(e) => {
            info!("Error processing download: {}", e);
            let error_message = format!("❌ Error: {}", e);
            bot.edit_message_text(msg.chat.id, processing_msg.id, error_message)
                .await?;
        }
    }

    Ok(())
}

async fn process_download(
    bot: &Bot,
    msg: &Message,
    url: &str,
    instagram_service: &InstagramService,
    rate_limiter: &RateLimiter,
) -> Result<i32> {
    // Block group chats
    if msg.chat.id.0 < 0 {
        return Err(anyhow::anyhow!("Group chats are not supported"));
    }

    // Check rate limit
    if !rate_limiter.check_rate_limit(msg.chat.id).await? {
        return Ok(-1);
    }
    info!("Parsing URL...");
    let parsed_url = parse_url(url)?;

    // Determine if it's a story URL
    let is_story = parsed_url.path().starts_with("/stories/");
    info!("URL type: {}", if is_story { "Story" } else { "Post" });

    let media_info = if is_story {
        info!("Processing story...");
        // For stories, we need both username and story ID
        let path_segments: Vec<_> = parsed_url
            .path_segments()
            .ok_or_else(|| anyhow::anyhow!("Invalid URL"))?
            .collect();

        match path_segments.as_slice() {
            ["stories", username, story_id] => {
                let story_identifier = format!("{}_{}", username, story_id);
                info!("Story identifier: {}", story_identifier);
                // TODO: why clone?
                match instagram_service.clone().get_story_info(&story_identifier).await {
                    Ok(info) => info,
                    Err(e) if e.to_string().contains("Please login first") => {
                        return Err(anyhow!("{}", login::get_login_instructions()));
                    }
                    Err(e) => return Err(e),
                }
            }
            _ => return Err(anyhow::anyhow!("Invalid story URL format")),
        }
    } else {
        info!("Processing regular post...");
        let shortcode = instagram_service.extract_shortcode(&parsed_url)?;

        // Check cache for non-story content
        if let Some(cached) = CacheService::get_media_info(&shortcode).await? {
            info!("Found in cache");
            cached
        } else {
            info!("Fetching from Instagram...");
            let media_info = instagram_service.get_media_info(&shortcode).await?;
            CacheService::set_media_info(&shortcode, &media_info).await?;
            media_info
        }
    };

    send_media(bot, msg, &media_info).await?;
    Ok(1)
}

async fn send_media(bot: &Bot, msg: &Message, media_info: &MediaInfo) -> Result<()> {
    info!("Sending media...");
    match media_info.media_type {
        MediaType::Image => {
            let parsed_url = parse_url(&media_info.url)?;
            bot.send_photo(msg.chat.id, InputFile::url(parsed_url)).await?;
        }
        MediaType::Video => {
            let parsed_url = parse_url(&media_info.url)?;
            bot.send_video(msg.chat.id, InputFile::url(parsed_url)).await?;
        }
        MediaType::Carousel => {
            info!("Handling multiple media items...");
            for item in media_info.carousel_items.clone() {
                match item.media_type {
                    MediaType::Image => {
                        let parsed_url = parse_url(&item.url)?;
                        bot.send_photo(msg.chat.id, InputFile::url(parsed_url)).await?;
                    }
                    MediaType::Video => {
                        let parsed_url = parse_url(&item.url)?;
                        bot.send_video(msg.chat.id, InputFile::url(parsed_url)).await?;
                    }
                    _ => continue,
                }
            }
        }
        MediaType::Story => {
            info!("Story info: {:?}", media_info);
            // TODO: implement this after login is implemented
        }
    };
    Ok(())
}
