use teloxide::{prelude::*, types::ParseMode, utils::markdown::escape};

pub async fn handle(bot: Bot, msg: Message) -> ResponseResult<()> {
    let welcome_text = format!(
        "👋 Hi {}!\n\n\
        I can help you download media from Instagram.\n\n\
        Use /help to see available commands.",
        escape(&msg.from.map(|user| user.first_name.clone()).unwrap_or_default())
    );

    bot.send_message(msg.chat.id, welcome_text)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

    Ok(())
}
