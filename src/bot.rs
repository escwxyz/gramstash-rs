use teloxide::prelude::*;
use teloxide::Bot;

use crate::commands::{answer, Command};
use crate::services::downloader::DownloaderService;
use crate::utils::error::BotError;

pub struct BotService {
    pub bot: Bot,
    pub downloader: DownloaderService,
}

impl BotService {
    pub async fn start(&self) -> Result<(), BotError> {
        let bot = self.bot.clone();
        let downloader = self.downloader.clone();

        Command::repl(bot, move |bot, msg, cmd| answer(bot, msg, cmd, downloader.clone())).await;

        Ok(())
    }
}
