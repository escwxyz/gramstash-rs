use teloxide::{prelude::*, types::ParseMode};

pub async fn handle(bot: Bot, msg: Message) -> ResponseResult<()> {
    let help_text = "🔍 *Available Commands*\n\n\
        /start \\- Start the bot\n\
        /help \\- Show this help message\n\
        /download <url> \\- Download media from Instagram\n\n\
        *Examples:*\n\
        `/download https://www.instagram.com/p/ABC123/`\n\
        `/download https://www.instagram.com/reel/XYZ789/`";

    bot.send_message(msg.chat.id, help_text)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

    Ok(())
}