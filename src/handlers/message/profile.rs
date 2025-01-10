use teloxide::{
    dispatching::dialogue::ErasedStorage,
    payloads::{EditMessageTextSetters, SendMessageSetters},
    prelude::{Dialogue, Requester},
    types::{Message, MessageId, ParseMode},
    Bot,
};

use crate::{
    services::{dialogue::DialogueState, middleware::process_instagram_username},
    state::AppState,
    utils::{
        error::{BotError, HandlerResult},
        keyboard, validate_instagram_password,
    },
};

pub async fn handle_message_profile_menu(bot: Bot, msg: Message) -> HandlerResult<()> {
    let text = t!("messages.profile_menu").to_string();
    bot.send_message(msg.chat.id, text)
        .reply_markup(keyboard::ProfileMenu::get_profile_menu_inline_keyboard())
        .await?;

    Ok(())
}

pub(super) async fn handle_message_username(
    bot: Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: Message,
    prompt_msg_id: MessageId,
) -> HandlerResult<()> {
    bot.delete_message(msg.chat.id, prompt_msg_id).await?;

    let username_input = msg
        .text()
        .ok_or_else(|| BotError::InvalidState("No username provided".into()))?;

    info!("username_input: {:?}", username_input);

    let validating_msg = bot
        .send_message(msg.chat.id, "🔑 Validating username ...")
        .parse_mode(ParseMode::Html)
        .await?;

    let username = match process_instagram_username(&username_input) {
        Ok(username) => username,
        Err(_) => {
            bot.edit_message_text(
                msg.chat.id,
                validating_msg.id,
                format!(
                    "❌ Invalid username {}. \n\nPlease input a valid username again.",
                    &username_input
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
            dialogue.update(DialogueState::AwaitingUsername(msg.id)).await?;
            return Ok(());
        }
    };

    let session_service = AppState::get()?.session.lock().await;

    let telegram_user_id = msg.clone().from.unwrap().id.to_string();

    // Check if there's a valid session for this user
    let session_msg = bot
        .edit_message_text(msg.chat.id, validating_msg.id, "🔑 Validating session ...")
        .parse_mode(ParseMode::Html)
        .await?;

    if session_service.validate_session(&telegram_user_id).await? {
        // If session exists and is valid, check if it matches the provided username
        if let Some(stored_session) = session_service.get_session(&telegram_user_id).await? {
            if let Some(session_data) = &stored_session.session_data {
                if session_data.username == Some(username.clone()) {
                    bot.edit_message_text(
                        msg.chat.id,
                        session_msg.id,
                        "✅ Successfully logged in with existing session!\n\
                        What would you like to do?",
                    )
                    .parse_mode(ParseMode::Html)
                    .reply_markup(keyboard::MainMenu::get_inline_keyboard())
                    .await?;
                    return Ok(());
                }
            }
        }
    }

    let password_msg = bot
        .edit_message_text(
            msg.chat.id,
            session_msg.id,
            format!(
                "🔑 Session is invalid, {}.\n\nPlease login again and input your Instagram password.\n\nNote: Your password will never be stored or used for anything else.",
                username
            ),
        )
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard::LoginDialogue::get_cancel_auth_keyboard())
        .await?;

    dialogue
        .update(DialogueState::AwaitingPassword {
            username: username.to_string(),
            prompt_msg_id: password_msg.id,
        })
        .await?;
    // TODO: test this
    bot.delete_message(msg.chat.id, msg.id).await?;

    Ok(())
}

pub(super) async fn handle_message_password(
    bot: Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: Message,
    (username, prompt_msg_id): (String, MessageId),
) -> HandlerResult<()> {
    bot.delete_message(msg.chat.id, prompt_msg_id).await?;

    let password = msg
        .text()
        .ok_or_else(|| BotError::InvalidState("No password provided".into()))?;

    if !validate_instagram_password(&password) {
        bot.delete_message(msg.chat.id, msg.id).await?;
        bot.send_message(msg.chat.id, "❌ Invalid password. Please try again.")
            .parse_mode(ParseMode::Html)
            .await?;

        dialogue
            .update(DialogueState::AwaitingPassword {
                username,
                prompt_msg_id: msg.id,
            })
            .await?;
        return Ok(());
    }

    // IMPORTANT: Delete the password message immediately
    bot.delete_message(msg.chat.id, msg.id).await?;

    let status_msg = bot
        .send_message(msg.chat.id, "🔄 Logging in to Instagram ...")
        .parse_mode(ParseMode::Html)
        .await?;

    let state = AppState::get()?;

    let mut instagram_service = state.instagram.lock().await;
    let mut session_service = state.session.lock().await;

    match instagram_service.login(&username, &password).await {
        Ok(session_data) => {
            let telegram_user_id = msg.from.unwrap().id.to_string();

            session_service.sync_session(&telegram_user_id, session_data).await?;

            bot.edit_message_text(
                msg.chat.id,
                status_msg.id,
                "✅ Login successful! You can now download stories.\n\n\
                 Note: This session won't expire until you logout manually.",
            )
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard::MainMenu::get_inline_keyboard())
            .await?;
        }
        Err(e) => {
            let msg = bot
                .edit_message_text(
                    msg.chat.id,
                    status_msg.id,
                    format!("❌ Login failed: {}\n\nPlease try again by inputing your username.", e),
                )
                .parse_mode(ParseMode::Html)
                .await?;

            dialogue.update(DialogueState::AwaitingUsername(msg.id)).await?;

            return Ok(());
        }
    }

    Ok(())
}
