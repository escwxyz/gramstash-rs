use crate::services::cache::CacheService;
use crate::services::instagram::{InstagramService, MediaInfo, MediaType};
use crate::services::ratelimiter::RateLimiter;
use crate::utils::parse_url;
use anyhow::Result;
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
    // Check rate limit
    if !rate_limiter.check_rate_limit(msg.chat.id).await? {
        return Ok(-1);
    }
    info!("Parsing URL...");
    let parsed_url = parse_url(url)?;
    info!("Extracting shortcode...");
    let shortcode = instagram_service.extract_shortcode(&parsed_url)?;
    info!("Checking cache...");
    let cached_media_info = CacheService::get_media_info(&shortcode).await?;

    info!("Cached media info: {:?}", cached_media_info);

    let media_info = match cached_media_info {
        Some(media_info) => {
            info!("Media info found in cache, sending media...");
            media_info
        }
        None => {
            info!("Media info not found in cache, fetching from Instagram...");
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
        // MediaType::Video => {
        //     info!("Checking if video is large...");
        //     if media_info.file_size > 50_000_000 {
        //         info!("Video is large, sending download link...");
        //         bot.send_message(
        //             msg.chat.id,
        //             "⚠️ This video is larger than 50MB. Sending download link instead.",
        //         )
        //         .await?;
        //         bot.send_message(msg.chat.id, &media_info.url).await?;
        //         return Ok(());
        //     }
        //     bot.send_video(msg.chat.id, InputFile::file(media_info.url)).await?;
        // }
        // MediaType::Carousel => {
        //     info!("Handling multiple media items...");
        //     for item in media_info.carousel_items {
        //         match item.media_type {
        //             MediaType::Image => {
        //                 bot.send_photo(msg.chat.id, InputFile::file(media_info.url)).await?;
        //             }
        //             MediaType::Video => {
        //                 if item.file_size <= 50_000_000 {
        //                     bot.send_video(msg.chat.id, InputFile::file(media_info.url)).await?;
        //                 } else {
        //                     bot.send_message(msg.chat.id, &item.url).await?;
        //                 }
        //             }
        //             _ => continue,
        //         }
        //     }
        // }
        _ => {
            info!("Unsupported media type: {:?}", media_info.media_type);
        }
    };
    Ok(())
}
