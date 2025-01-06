use teloxide::{dispatching::dialogue::ErasedStorage, prelude::*, types::MessageId};

use crate::{
    services::{dialogue::DialogueState, instagram::InstagramService},
    utils::{
        error::{BotError, HandlerResult},
        keyboard,
    },
};

pub async fn handle_logout(
    bot: Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: Message,
    prompt_msg_id: MessageId,
) -> HandlerResult<()> {
    bot.delete_message(msg.chat.id, prompt_msg_id).await?;

    let mut instagram_service = InstagramService::new();
    if let Err(e) = instagram_service.logout().await {
        bot.send_message(msg.chat.id, format!("❌ Logout failed: {}", e))
            .reply_markup(keyboard::get_settings_keyboard())
            .await?;
        // TODO: Handle this error
        return Ok(());
    }

    dialogue
        .update(DialogueState::ConfirmLogout)
        .await
        .map_err(|e| BotError::DialogueError(e.to_string()))?;

    bot.send_message(
        msg.chat.id,
        "✅ Logged out successfully.\n\nWhat would you like to do next?",
    )
    .reply_markup(keyboard::get_main_menu_keyboard())
    .await?;

    Ok(())
}
