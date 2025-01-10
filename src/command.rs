use teloxide::{macros::BotCommands, types::BotCommand};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub enum Command {
    #[command(description = "Start the bot and show main menu")]
    Start,
    #[command(description = "Change language")]
    Language,
    #[command(description = "Show help message")]
    Help,
    #[command(description = "Admin only - show statistics")]
    Stats,
    #[command(description = "Admin only - show system status")]
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
