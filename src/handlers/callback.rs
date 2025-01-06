use crate::{
    services::dialogue::DialogueState,
    utils::{
        error::{BotError, HandlerResult},
        keyboard,
    },
};
use teloxide::{
    dispatching::dialogue::ErasedStorage,
    prelude::*,
    types::{CallbackQuery, ParseMode},
};

use super::message::download::handle_download;

pub async fn handle_callback(
    bot: Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    q: CallbackQuery,
) -> HandlerResult<()> {
    let data = q
        .data
        .ok_or_else(|| BotError::InvalidState("No callback data".into()))?;

    let message = q.message.ok_or_else(|| BotError::InvalidState("No message".into()))?;

    match data.as_str() {
        "download_menu" => {
            bot.edit_message_text(message.chat().id, message.id(), "What would you like to download?")
                .reply_markup(keyboard::get_download_menu_keyboard())
                .await
                .map_err(|e| BotError::Other(e.into()))?;
        }
        "download_post" => {
            let msg = bot
                .edit_message_text(
                    message.chat().id,
                    message.id(),
                    "Please send me the Instagram post or reel URL you want to download.",
                )
                .await
                .map_err(|e| BotError::Other(e.into()))?;

            dialogue
                .update(DialogueState::AwaitingPostLink(msg.id))
                .await
                .map_err(|e| BotError::DialogueError(e.to_string()))?;
        }
        "download_story" => {
            let msg = bot
                .edit_message_text(
                    message.chat().id,
                    message.id(),
                    "Please send me the Instagram story URL you want to download.",
                )
                .await
                .map_err(|e| BotError::Other(e.into()))?;

            dialogue
                .update(DialogueState::AwaitingStoryLink(msg.id))
                .await
                .map_err(|e| BotError::DialogueError(e.to_string()))?;
        }

        "confirm" => {
            if let Some(DialogueState::ConfirmDownload { content }) = dialogue
                .get()
                .await
                .map_err(|e| BotError::DialogueError(e.to_string()))?
            {
                handle_download(bot.clone(), message, content).await?;
                dialogue
                    .update(DialogueState::Start)
                    .await
                    .map_err(|e| BotError::DialogueError(e.to_string()))?;
            }
        }

        "cancel" => {
            dialogue
                .update(DialogueState::Start)
                .await
                .map_err(|e| BotError::DialogueError(e.to_string()))?;

            bot.edit_message_text(
                message.chat().id,
                message.id(),
                "Download cancelled. What would you like to do?",
            )
            .reply_markup(keyboard::get_main_menu_keyboard())
            .await?;
        }

        // ------------
        "settings_menu" => {
            info!("Showing settings menu");
            bot.edit_message_text(message.chat().id, message.id(), "⚙️ Settings")
                .reply_markup(keyboard::get_settings_keyboard())
                .await
                .map_err(|e| BotError::Other(e.into()))?;
        }
        "help_menu" => {
            info!("Showing help menu");

            bot.edit_message_text(
                message.chat().id,
                message.id(),
                "ℹ️ Help and Information\n\n\
                 /start \\- Start the bot\n\
                 /help \\- Show this help message",
            )
            .parse_mode(ParseMode::MarkdownV2)
            .await
            .map_err(|e| BotError::Other(e.into()))?;
        }
        "main_menu" => {
            info!("Exiting dialogue");

            dialogue
                .exit()
                .await
                .map_err(|e| BotError::DialogueError(e.to_string()))?;

            bot.edit_message_text(message.chat().id, message.id(), "Please select an option:")
                .reply_markup(keyboard::get_main_menu_keyboard())
                .await
                .map_err(|e| BotError::Other(e.into()))?;
        }
        "login" => {
            // Only ask for username first
            let username_msg = bot
                .edit_message_text(message.chat().id, message.id(), "Please input your Instagram username")
                .await
                .map_err(|e| BotError::Other(e.into()))?;

            // Update state to await username
            dialogue
                .update(DialogueState::AwaitingUsername(username_msg.id))
                .await
                .map_err(|e| BotError::DialogueError(e.to_string()))?;
        }
        _ => {
            bot.answer_callback_query(&q.id)
                .text("Unknown command")
                .await
                .map_err(|e| BotError::Other(e.into()))?;
        }
    }

    bot.answer_callback_query(&q.id)
        .await
        .map_err(|e| BotError::Other(e.into()))?;

    Ok(())
}
