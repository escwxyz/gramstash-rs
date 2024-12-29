use teloxide::{macros::BotCommands, prelude::ResponseResult, types::Message, Bot};

use crate::{
    handlers::{download, help, start},
    services::{instagram::InstagramService, ratelimiter::RateLimiter},
    state::AppState,
};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Supported commands:")]
pub enum Command {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Show help message")]
    Help,
    #[command(description = "Download media from Instagram. Usage: /download <url>")]
    Download { url: String },
}

pub async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let state = AppState::get();

    let rate_limiter = RateLimiter::new();

    let instagram_api_endpoint = state.config.instagram.api_endpoint.clone();
    let instagram_doc_id = state.config.instagram.doc_id.clone();

    let instagram_service = InstagramService::new(instagram_api_endpoint, instagram_doc_id);

    match cmd {
        Command::Start => start::handle(bot, msg).await,
        Command::Help => help::handle(bot, msg).await,
        Command::Download { url } => download::handle(bot, msg, url, &instagram_service, &rate_limiter).await,
    }
}
