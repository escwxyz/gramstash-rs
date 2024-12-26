use teloxide::{prelude::*, types::ParseMode, utils::markdown::escape};

pub async fn handle(bot: Bot, msg: Message) -> ResponseResult<()> {
    let user_name = msg.from
        .map(|user| escape(&user.first_name))
        .unwrap_or_else(|| escape("there"));

    let welcome_text = format!(
        "👋 Hi {}\n\n\
        I can help you download media from Instagram\\.\n\n\
        Use /help to see available commands\\.",
        user_name
    );

    bot.send_message(msg.chat.id, welcome_text)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

    Ok(())
}
