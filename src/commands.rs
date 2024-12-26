use teloxide::{macros::BotCommands, prelude::ResponseResult, types::Message, Bot};

use crate::{
    handlers::{download, help, start},
    services::downloader::DownloaderService,
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

pub async fn answer(bot: Bot, msg: Message, cmd: Command, downloader: DownloaderService) -> ResponseResult<()> {
    match cmd {
        Command::Start => start::handle(bot, msg).await,
        Command::Help => help::handle(bot, msg).await,
        Command::Download { url } => download::handle(bot, msg, url, &downloader).await,
    }
}
