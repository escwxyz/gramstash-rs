use chrono::Utc;
use teloxide::{
    adaptors::Throttle,
    dispatching::dialogue::ErasedStorage,
    payloads::EditMessageTextSetters,
    prelude::{Dialogue, Requester},
    types::{Message, MessageId},
    Bot,
};

use crate::{
    context::UserContext,
    error::{BotError, HandlerResult},
    handler::keyboard::{get_cancel_auth_keyboard, get_main_menu_keyboard},
    platform::{
        instagram::{process_instagram_username, validate_instagram_password},
        Platform,
    },
    service::{dialogue::model::DialogueState, Credentials, PlatformSessionData, Session, SessionStatus},
    state::AppState,
    utils::reconstruct_raw_text,
};

pub(super) async fn handle_message_username(
    bot: Throttle<Bot>,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: Message,
    prompt_msg_id: MessageId,
) -> HandlerResult<()> {
    info!("handle_message_username");

    bot.delete_message(msg.chat.id, prompt_msg_id).await?;

    let entities = msg.parse_entities();

    let text = msg.text().unwrap();

    let raw_text = if let Some(entities) = entities {
        reconstruct_raw_text(text, &entities)
    } else {
        text.to_string()
    };

    info!("raw_text: {:?}", raw_text);

    let username_input = msg.text().unwrap();

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

    let telegram_user_id = UserContext::global().user_id().to_string();

    let state = AppState::get()?;
    let session_service = state.service_registry.session;

    // First check if user is already authenticated
    let is_authenticated = session_service
        .is_authenticated(&telegram_user_id, &Platform::Instagram)
        .await
        .unwrap_or(false);

    if is_authenticated {
        // Then check if the username matches current session
        // TODO
        if let Some(session) = session_service
            .get_cached_session(&telegram_user_id, &Platform::Instagram)
            .await
            .unwrap()
        {
            if let Some(PlatformSessionData::Instagram(instagram_session_data)) = session.get_platform_data() {
                if instagram_session_data.username == username {
                    bot.edit_message_text(
                        msg.chat.id,
                        session_msg.id,
                        t!("messages.profile.username.validating_session_success"),
                    )
                    .reply_markup(get_main_menu_keyboard())
                    .await?;
                    return Ok(());
                }
            }
        }
    } else {
        session_service
            .remove_cached_session(&telegram_user_id, &Platform::Instagram)
            .await?;
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
        .reply_markup(get_cancel_auth_keyboard())
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
) -> HandlerResult<()> {
    bot.delete_message(msg.chat.id, prompt_msg_id).await?;

    let text = msg.text().unwrap();

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
    let telegram_user_id = UserContext::global().user_id().to_string();

    let auth_service = state.service_registry.auth.lock().await;

    match auth_service
        .login(&Credentials {
            indentifier: username,
            password: raw_text,
            platform: Platform::Instagram,
            two_factor_token: None,
        })
        .await
    {
        Ok(session_data) => {
            let session = Session {
                telegram_user_id: telegram_user_id.to_string(),
                platform: Platform::Instagram,
                status: SessionStatus::Active,
                last_accessed: Utc::now(),
                last_refresh: Utc::now(),
                session_data: Some(session_data),
            };

            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            if auth_service.verify_session(&session).await? {
                state
                    .service_registry
                    .session
                    .save_cached_session(session, &Platform::Instagram)
                    .await?;

                bot.edit_message_text(
                    msg.chat.id,
                    status_msg.id,
                    t!("messages.profile.password.login_success"),
                )
                .reply_markup(get_main_menu_keyboard())
                .await?;
            } else {
                let msg = bot
                    .edit_message_text(
                        msg.chat.id,
                        status_msg.id,
                        t!(
                            "messages.profile.password.login_failed",
                            error = "I don't know".to_string() // TODO
                        ),
                    )
                    .await?;

                dialogue
                    .update(DialogueState::AwaitingUsername(msg.id))
                    .await
                    .map_err(|e| BotError::DialogueStateError(e.to_string()))?;
            }
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
