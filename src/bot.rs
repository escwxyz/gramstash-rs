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

        let handler = Update::filter_message()
            .branch(
                dptree::filter(|msg: Message| {
                    if let Some(text) = msg.text() {
                        if text.starts_with('/') {
                            let parts: Vec<&str> = text.split_whitespace().collect();
                            match parts[0] {
                                "/login" if parts.len() != 3 => return true,
                                "/download" if parts.len() != 2 => return true,
                                _ => return false,
                            }
                        }
                    }
                    false
                })
                .endpoint(handle_invalid_command),
            )
            .branch(dptree::entry().filter_command::<Command>().endpoint(answer));

        Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
        Ok(())
    }
}

async fn handle_invalid_command(bot: Bot, msg: Message) -> ResponseResult<()> {
    let command = msg.text().unwrap_or("").split_whitespace().next().unwrap_or("");

    let error_msg = match command {
        "/login" => format!(
            "⚠️ Invalid login format.\n\nUsage: {}\n\nExample: /login myusername mypassword",
            Command::Login {
                username: String::new(),
                password: String::new()
            }
            .usage()
        ),
        "/download" => format!(
            "⚠️ Please provide a URL.\n\nUsage: {}",
            Command::Download { url: String::new() }.usage()
        ),
        _ => "❌ Invalid command format. Use /help to see available commands.".to_string(),
    };

    bot.send_message(msg.chat.id, error_msg).await?;
    Ok(())
}
