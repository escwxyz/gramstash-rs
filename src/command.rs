use teloxide::{
    macros::BotCommands,
    payloads::SetMyCommandsSetters,
    prelude::Requester,
    types::{BotCommand, BotCommandScope, ChatId, Recipient},
    Bot,
};

use crate::error::HandlerResult;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum UserCommand {
    Start,
    Language,
    Help,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    Start,
    Language,
    Help,
    Stats,
    Status,
}

impl Command {
    pub fn user_commands() -> Vec<BotCommand> {
        vec![
            BotCommand::new("start", t!("commands.description.start")),
            BotCommand::new("help", t!("commands.description.help")),
            BotCommand::new("language", t!("commands.description.language")),
        ]
    }

    #[allow(unused)]
    pub fn admin_commands() -> Vec<BotCommand> {
        vec![
            BotCommand::new("start", t!("commands.description.start")),
            BotCommand::new("help", t!("commands.description.help")),
            BotCommand::new("language", t!("commands.description.language")),
            BotCommand::new("stats", t!("commands.description.stats")),
            BotCommand::new("status", t!("commands.description.status")),
        ]
    }
}

pub async fn setup_user_commands(bot: &Bot) -> HandlerResult<()> {
    bot.delete_my_commands().await?;
    bot.set_my_commands(Command::user_commands()).await?;
    Ok(())
}
#[allow(unused)]
async fn setup_admin_commands(bot: &Bot, chat_id: ChatId) -> HandlerResult<()> {
    bot.delete_my_commands().await?;
    bot.set_my_commands(Command::admin_commands())
        .scope(BotCommandScope::Chat {
            chat_id: Recipient::Id(chat_id),
        })
        .await?;
    Ok(())
}

#[cfg(not(test))]
pub async fn setup_commands(bot: &Bot, is_admin: bool, chat_id: ChatId) -> HandlerResult<()> {
    if is_admin {
        setup_admin_commands(bot, chat_id).await?;
    } else {
        setup_user_commands(bot).await?;
    }
    Ok(())
}

#[cfg(test)]
pub async fn setup_commands(_bot: &Bot, _is_admin: bool, _chat_id: ChatId) -> HandlerResult<()> {
    Ok(())
}
