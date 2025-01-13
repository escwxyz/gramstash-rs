use teloxide::{
    adaptors::DefaultParseMode,
    macros::BotCommands,
    payloads::SetMyCommandsSetters,
    prelude::Requester,
    types::{BotCommand, BotCommandScope, ChatId, Recipient},
    Bot,
};

use crate::{config::AdminConfig, error::HandlerResult};

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
            BotCommand::new("start", "Start the bot and show main menu"),
            BotCommand::new("help", "Show help message"),
            BotCommand::new("language", "Change language"),
        ]
    }

    #[allow(unused)]
    pub fn admin_commands() -> Vec<BotCommand> {
        vec![
            BotCommand::new("start", "Start the bot and show main menu"),
            BotCommand::new("help", "Show help message"),
            BotCommand::new("stats", "Show statistics"),
            BotCommand::new("status", "Show system status"),
        ]
    }
}

pub async fn setup_commands(bot: &DefaultParseMode<Bot>, admin_config: &AdminConfig) -> HandlerResult<()> {
    info!("Setting up bot commands...");

    bot.delete_my_commands().await?;

    if let Err(_) = bot
        .set_my_commands(Command::admin_commands())
        .scope(BotCommandScope::Chat {
            chat_id: Recipient::Id(ChatId(admin_config.telegram_user_id.0 as i64)),
        })
        .await
    {
        // If we can't set admin commands, set user commands
        match bot.set_my_commands(Command::user_commands()).await {
            Ok(_) => info!("Successfully set up user bot commands"),
            Err(e) => {
                error!("Failed to set bot commands: {:?}", e);
                return Err(e.into());
            }
        }
    } else {
        info!("Successfully set up admin bot commands");
    }

    Ok(())
}
