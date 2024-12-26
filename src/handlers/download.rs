use crate::services::downloader::DownloaderService;
use crate::services::instagram::{InstagramService, MediaType};
use crate::utils::error::BotError;
use teloxide::prelude::*;
use teloxide::types::InputFile;

pub async fn handle(bot: Bot, msg: Message, url: String, downloader: &DownloaderService) -> ResponseResult<()> {
    // Send processing message
    let processing_msg = bot.send_message(msg.chat.id, "⏳ Processing your request...").await?;

    // Validate URL
    if !url.contains("instagram.com") {
        bot.edit_message_text(
            msg.chat.id,
            processing_msg.id,
            "❌ Please provide a valid Instagram URL",
        )
        .await?;
        return Ok(());
    }

    // Initialize Instagram service
    let instagram_service = InstagramService::new();

    // Process the download
    match process_download(&bot, &msg, &instagram_service, &downloader, &url).await {
        Ok(_) => {
            bot.edit_message_text(msg.chat.id, processing_msg.id, "✅ Download completed!")
                .await?;
        }
        Err(e) => {
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
    instagram_service: &InstagramService,
    downloader: &DownloaderService,
    url: &str,
) -> Result<(), BotError> {
    // Extract media info
    let media_info = instagram_service.get_media_info(url).await?;

    // Send appropriate message based on media type
    match media_info.media_type {
        MediaType::Image => {
            let file_path = downloader.download_media(&media_info.url, msg.chat.id.0).await?;

            bot.send_photo(msg.chat.id, InputFile::file(file_path)).await?;
        }
        MediaType::Video => {
            // Send a message if video is large
            if media_info.file_size > 50_000_000 {
                bot.send_message(
                    msg.chat.id,
                    "⚠️ This video is larger than 50MB. Sending download link instead.",
                )
                .await?;
                bot.send_message(msg.chat.id, &media_info.url).await?;
                return Ok(());
            }
            let file_path = downloader.download_media(&media_info.url, msg.chat.id.0).await?;
            bot.send_video(msg.chat.id, InputFile::file(file_path)).await?;
        }
        MediaType::Carousel => {
            // Handle multiple media items
            for item in media_info.carousel_items {
                match item.media_type {
                    MediaType::Image => {
                        let file_path = downloader.download_media(&item.url, msg.chat.id.0).await?;
                        bot.send_photo(msg.chat.id, InputFile::file(file_path)).await?;
                    }
                    MediaType::Video => {
                        if item.file_size <= 50_000_000 {
                            let file_path = downloader.download_media(&item.url, msg.chat.id.0).await?;
                            bot.send_video(msg.chat.id, InputFile::file(file_path)).await?;
                        } else {
                            bot.send_message(msg.chat.id, &item.url).await?;
                        }
                    }
                    _ => continue,
                }
            }
        }
    }

    Ok(())
}
