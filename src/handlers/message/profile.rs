use teloxide::{
    adaptors::Throttle,
    dispatching::dialogue::ErasedStorage,
    payloads::EditMessageTextSetters,
    prelude::{Dialogue, Requester},
    types::{Message, MessageId},
    Bot,
};

use crate::{
    error::{BotError, HandlerResult, MiddlewareError, ServiceError},
    handlers::RequestContext,
    services::{
        auth::Credentials,
        dialogue::DialogueState,
        middleware::{process_instagram_username, reconstruct_raw_text},
    },
    state::AppState,
    utils::{keyboard, validate_instagram_password},
};

pub(super) async fn handle_message_username(
    bot: Throttle<Bot>,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: Message,
    prompt_msg_id: MessageId,
    ctx: RequestContext,
) -> HandlerResult<()> {
    info!("handle_message_username");

    bot.delete_message(msg.chat.id, prompt_msg_id).await?;

    let entities = msg.parse_entities();

    let text = msg.text().ok_or_else(|| {
        BotError::ServiceError(ServiceError::Middleware(MiddlewareError::Other(
            "No username provided".into(),
        )))
    })?;

    let raw_text = if let Some(entities) = entities {
        reconstruct_raw_text(text, &entities)
    } else {
        text.to_string()
    };

    info!("raw_text: {:?}", raw_text);

    let username_input = msg.text().ok_or_else(|| {
        BotError::ServiceError(ServiceError::Middleware(MiddlewareError::Other(
            "No username provided".into(),
        )))
    })?;

    info!("username_input: {:?}", username_input);

    let validating_msg = bot
        .send_message(msg.chat.id, t!("messages.profile.username.validating"))
        .await?;

    let username = match process_instagram_username(&raw_text) {
        Ok(username) => username,
        Err(_) => {
            bot.edit_message_text(
                msg.chat.id,
                validating_msg.id,
                t!(
                    "messages.profile.username.invalid",
                    username = username_input.to_string()
                ),
            )
            .await?;
            dialogue
                .update(DialogueState::AwaitingUsername(msg.id))
                .await
                .map_err(|e| BotError::DialogueStateError(e.to_string()))?;
            return Ok(());
        }
    };

    // Check if there's a valid session for this user
    let session_msg = bot
        .edit_message_text(
            msg.chat.id,
            validating_msg.id,
            t!("messages.profile.username.validating_session"),
        )
        .await?;

    let telegram_user_id = ctx.telegram_user_id.to_string();

    // First check if user is already authenticated
    let is_authenticated = AppState::get()?
        .session
        .is_authenticated(&telegram_user_id)
        .await
        .unwrap_or(false);

    if is_authenticated {
        // Then check if the username matches current session
        if let Some(session) = AppState::get()?.session.session_cache.get(&telegram_user_id) {
            if let Some(session_data) = &session.session_data {
                if session_data.username == username {
                    bot.edit_message_text(
                        msg.chat.id,
                        session_msg.id,
                        t!("messages.profile.username.validating_session_success"),
                    )
                    .reply_markup(keyboard::MainMenu::get_inline_keyboard())
                    .await?;
                    return Ok(());
                }
            }
        }
    } else {
        AppState::get()?.session.invalidate_cache(&telegram_user_id);
    }

    let password_msg = bot
        .edit_message_text(
            msg.chat.id,
            session_msg.id,
            t!(
                "messages.profile.username.invalid_session",
                username = username.to_string()
            ),
        )
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
    bot: Throttle<Bot>,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: Message,
    (username, prompt_msg_id): (String, MessageId),
    ctx: RequestContext,
) -> HandlerResult<()> {
    bot.delete_message(msg.chat.id, prompt_msg_id).await?;

    let text = msg.text().ok_or_else(|| {
        BotError::ServiceError(ServiceError::Middleware(MiddlewareError::Other(
            "No password provided".into(),
        )))
    })?;

    let entities = msg.parse_entities();

    let raw_text = if let Some(entities) = entities {
        reconstruct_raw_text(text, &entities)
    } else {
        text.to_string()
    };

    info!("raw_text: {:?}", raw_text);

    if !validate_instagram_password(&raw_text) {
        bot.delete_message(msg.chat.id, msg.id).await?;
        bot.send_message(msg.chat.id, t!("messages.profile.password.invalid"))
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
        .send_message(msg.chat.id, t!("messages.profile.password.logging_in"))
        .await?;

    let state = AppState::get()?;
    let telegram_user_id = ctx.telegram_user_id.to_string();

    let auth_service = state.auth.lock().await;

    match auth_service
        .login(Credentials {
            username,
            password: raw_text,
        })
        .await
    {
        Ok(session_data) => {
            state.session.save_session(&telegram_user_id, session_data).await?;

            bot.edit_message_text(
                msg.chat.id,
                status_msg.id,
                t!("messages.profile.password.login_success"),
            )
            .reply_markup(keyboard::MainMenu::get_inline_keyboard())
            .await?;
        }
        Err(e) => {
            let msg = bot
                .edit_message_text(
                    msg.chat.id,
                    status_msg.id,
                    t!("messages.profile.password.login_failed", error = e.to_string()),
                )
                .await?;

            dialogue
                .update(DialogueState::AwaitingUsername(msg.id))
                .await
                .map_err(|e| BotError::DialogueStateError(e.to_string()))?;

            return Ok(());
        }
    }

    Ok(())
}
