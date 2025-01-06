use crate::{
    services::{dialogue::DialogueState, instagram::InstagramService},
    utils::{
        error::{BotError, HandlerResult},
        keyboard, unescape_markdown,
    },
};
use teloxide::{dispatching::dialogue::ErasedStorage, prelude::*, types::MessageId};

pub async fn handle_username(
    bot: Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: Message,
    prompt_msg_id: MessageId,
) -> HandlerResult<()> {
    bot.delete_message(msg.chat.id, prompt_msg_id).await?;

    let username = msg
        .text()
        .ok_or_else(|| BotError::InvalidState("No username provided".into()))?;

    let username = unescape_markdown(username);

    info!("Processing login for username: {}", username);

    let password_msg = bot
        .send_message(
            msg.chat.id,
            "Please enter your Instagram password.\n\
             Note: Your password will be never stored or used for anything else.",
        )
        .await?;

    dialogue
        .update(DialogueState::AwaitingPassword {
            username: username.to_string(),
            prompt_msg_id: password_msg.id,
        })
        .await
        .map_err(|e| BotError::DialogueError(e.to_string()))?;

    bot.delete_message(msg.chat.id, msg.id).await?;

    Ok(())
}

pub async fn handle_password(
    bot: Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: Message,
    (username, prompt_msg_id): (String, MessageId),
) -> HandlerResult<()> {
    bot.delete_message(msg.chat.id, prompt_msg_id).await?;

    let password = msg
        .text()
        .ok_or_else(|| BotError::InvalidState("No password provided".into()))?;

    // IMPORTANT: Delete the password message
    bot.delete_message(msg.chat.id, msg.id).await?;

    let password = unescape_markdown(password);

    let status_msg = bot.send_message(msg.chat.id, "🔄 Logging in to Instagram...").await?;

    let instagram_service = InstagramService::new();
    match instagram_service.clone().login(&username, &password).await {
        Ok(_) => {
            dialogue
                .update(DialogueState::LoggedIn)
                .await
                .map_err(|e| BotError::DialogueError(e.to_string()))?;

            bot.edit_message_text(
                msg.chat.id,
                status_msg.id,
                "✅ Login successful! You can now download stories.\n\n\
                 Note: This session won't expire until you logout manually.",
            )
            .reply_markup(keyboard::get_main_menu_keyboard())
            .await?;
        }
        Err(e) => {
            let msg = bot
                .edit_message_text(
                    msg.chat.id,
                    status_msg.id,
                    format!("❌ Login failed: {}\n\nPlease try again by inputing your username.", e),
                )
                .reply_markup(keyboard::get_back_to_menu_keyboard())
                .await?;

            dialogue
                .update(DialogueState::AwaitingUsername(msg.id))
                .await
                .map_err(|e| BotError::DialogueError(e.to_string()))?;

            return Ok(());
        }
    }

    Ok(())
}
