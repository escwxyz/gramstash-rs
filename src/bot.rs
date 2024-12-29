use anyhow::Result;
use teloxide::prelude::*;
use teloxide::Bot;

use crate::commands::{answer, Command};

pub struct BotService {
    pub bot: Bot,
}

impl BotService {
    pub async fn start(&self) -> Result<()> {
        let bot = self.bot.clone();

        Command::repl(bot, move |bot, msg, cmd| answer(bot, msg, cmd)).await;

        Ok(())
    }
}
