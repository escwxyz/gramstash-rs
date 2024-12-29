use anyhow::Result;
use teloxide::prelude::*;
use teloxide::Bot;

use crate::commands::{answer, Command};
use crate::utils::error::BotError;

pub struct BotService {
    pub bot: Bot,
}

impl BotService {
    pub async fn start(&self) -> Result<(), BotError> {
        let bot = self.bot.clone();

        Command::repl(bot, move |bot, msg, cmd| answer(bot, msg, cmd)).await;

        Ok(())
    }
}
