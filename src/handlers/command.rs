use teloxide::{macros::BotCommands, types::Message, Bot};
use teloxide::{prelude::*, types::ParseMode, utils::markdown::escape};

use crate::utils::keyboard;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Supported commands:")]
pub enum Command {
    #[command(description = "Start the bot and show main menu")]
    Start,
    #[command(description = "Show help message")]
    Help,
}

pub async fn command_handler(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Start => handle_start(bot, msg).await,
        Command::Help => handle_help(bot, msg).await,
    }
}

pub async fn handle_start(bot: Bot, msg: Message) -> ResponseResult<()> {
    let user_name = msg
        .from
        .map(|user| escape(&user.first_name))
        .unwrap_or_else(|| escape("there"));

    let welcome_text = format!(
        "👋 Hi {user_name}\\!\n\n\
        Welcome to GramStash\\! I can help you download content from Instagram\\.\n\n\
        Please select an option below\\:",
        user_name = user_name
    );

    bot.send_message(msg.chat.id, welcome_text)
        .parse_mode(ParseMode::MarkdownV2)
        .reply_markup(keyboard::get_main_menu_keyboard())
        .await?;

    Ok(())
}

async fn handle_help(bot: Bot, msg: Message) -> ResponseResult<()> {
    let help_text = format!("This is a help message");

    bot.send_message(msg.chat.id, help_text)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

    Ok(())
}
