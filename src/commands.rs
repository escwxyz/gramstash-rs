use teloxide::{
    macros::BotCommands,
    prelude::ResponseResult,
    types::Message,
    Bot,
};

use crate::{
    handlers::{download, help, login, logout, start},
    services::{instagram::InstagramService, ratelimiter::RateLimiter},
};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Supported commands:", parse_with = "split")]
pub enum Command {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Show help message")]
    Help,
    #[command(description = "Download media from Instagram. Usage: /download <url>")]
    Download { url: String },
    #[command(description = "Login to Instagram. Usage: /login <username> <password>")]
    Login { username: String, password: String },
    #[command(description = "Logout from Instagram. Usage: /logout")]
    Logout,
}

impl Command {
    pub fn usage(&self) -> &'static str {
        match self {
            Command::Start => "/start",
            Command::Help => "/help",
            Command::Download { .. } => "/download <url>",
            Command::Login { .. } => "/login <username> <password>",
            Command::Logout => "/logout",
        }
    }
}

pub async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let rate_limiter = RateLimiter::new();

    let instagram_service = InstagramService::new();

    match cmd {
        Command::Start => start::handle(bot, msg).await,
        Command::Help => help::handle(bot, msg).await,
        Command::Download { url } => download::handle(bot, msg, url, &instagram_service, &rate_limiter).await,
        Command::Login { username, password } => login::handle(bot, msg, username, password, &instagram_service).await,
        Command::Logout => logout::handle(bot, msg, &instagram_service).await,
    }
}
