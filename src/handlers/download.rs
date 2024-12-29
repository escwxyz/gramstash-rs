use crate::services::instagram::{InstagramService, MediaType};
use crate::services::ratelimiter::RateLimiter;
use crate::utils::error::BotError;
use anyhow::Result;
use teloxide::prelude::*;
use teloxide::types::InputFile;
use url::Url;

pub async fn handle(
    bot: Bot,
    msg: Message,
    url: String,
    instagram_service: &InstagramService,
    rate_limiter: &RateLimiter,
) -> ResponseResult<()> {
    info!("Downloading media from {}", url);

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
    match process_download(&bot, &msg, &url, &instagram_service, &rate_limiter).await {
        Ok(_) => {
            info!("Download completed!");
            bot.edit_message_text(msg.chat.id, processing_msg.id, "✅ Download completed!")
                .await?;
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
) -> Result<()> {
    // Check rate limit
    if !rate_limiter.check_rate_limit(msg.chat.id).await? {
        bot.send_message(msg.chat.id, "Daily download limit reached. Try again tomorrow!")
            .await?;
        return Ok(());
    }

    // TODO: we need to check cache here before fetching from Instagram's API

    info!("Extracting media info...");
    let media_info = instagram_service.get_media_info(url).await?;

    info!("Sending appropriate message based on media type...");
    match media_info.media_type {
        MediaType::Image => {
            // downloader.process_media(&media_info, msg.chat.id.0).await?;
            let parsed_url =
                Url::parse(&media_info.url).map_err(|e| BotError::ParseError(format!("Invalid image URL: {}", e)))?;

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
        //     let media_info = downloader.download_media(&media_info, msg.chat.id.0).await?;
        //     bot.send_video(msg.chat.id, InputFile::file(media_info.url)).await?;
        // }
        // MediaType::Carousel => {
        //     info!("Handling multiple media items...");
        //     for item in media_info.carousel_items {
        //         match item.media_type {
        //             MediaType::Image => {
        //                 let media_info = downloader.download_media(&item, msg.chat.id.0).await?;
        //                 bot.send_photo(msg.chat.id, InputFile::file(media_info.url)).await?;
        //             }
        //             MediaType::Video => {
        //                 if item.file_size <= 50_000_000 {
        //                     let media_info = downloader.download_media(&item, msg.chat.id.0).await?;
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
    }

    Ok(())
}
