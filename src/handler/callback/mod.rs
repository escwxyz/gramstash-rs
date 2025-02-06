mod download;
mod language;
mod navigation;
mod profile;

use std::str::FromStr;

use crate::{
    context::UserContext,
    error::{BotError, HandlerResult},
    platform::Platform,
    service::dialogue::model::DialogueState,
    state::AppState,
};

use teloxide::{
    adaptors::Throttle,
    dispatching::{dialogue::ErasedStorage, UpdateHandler},
    prelude::*,
    types::CallbackQuery,
};

async fn handle_callback(
    bot: Throttle<Bot>,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    q: CallbackQuery,
) -> HandlerResult<()> {
    let data = q
        .data
        .ok_or_else(|| BotError::DialogueStateError("No callback data".into()))?;

    let app_state = AppState::get()?;
    let interaction = app_state.service_registry.interaction;

    let message: teloxide::types::MaybeInaccessibleMessage = q
        .message
        .ok_or_else(|| BotError::DialogueStateError("No message".into()))?;

    let telegram_user_id = UserContext::global().user_id().to_string();

    match data.as_str() {
        // download
        "select_platform_menu" => {
            interaction
                .set_last_interface(&telegram_user_id, "select_platform")
                .await?;

            download::handle_callback_select_platform(&bot, dialogue, message).await?
        }

        s if s.starts_with("platform:") => {
            let platform_str = s.split(":").nth(1).unwrap_or("instagram");

            interaction
                .set_last_interface(&telegram_user_id, &format!("platform:{}", platform_str))
                .await?;

            let platform = Platform::from_str(platform_str).unwrap_or_default();

            download::handle_callback_asking_for_download_link(&bot, dialogue, message, platform).await?
        }

        "confirm_download" => {
            interaction
                .set_last_interface(&telegram_user_id, "confirm_download")
                .await?;
            download::handle_callback_confirm_download(&bot, dialogue, message).await?
        }
        "cancel_download" => {
            interaction
                .set_last_interface(&telegram_user_id, "cancel_download")
                .await?;
            download::handle_callback_cancel_download(&bot, message).await?
        }

        // profile
        "profile_menu" => {
            interaction
                .set_last_interface(&telegram_user_id, "profile_menu")
                .await?;
            profile::handle_callback_profile_menu(&bot, message).await?
        }
        // "cancel_auth" => {
        //     interaction
        //         .set_last_interface(ctx.telegram_user_id.to_string().as_str(), "cancel_auth")
        //         .await?;
        //     profile::handle_callback_profile_menu(&bot, message).await?
        // }
        // "auth_login" => {
        //     interaction
        //         .set_last_interface(ctx.telegram_user_id.to_string().as_str(), "auth_login")
        //         .await?;
        //     profile::handle_callback_auth_login(&bot, dialogue, message).await?
        // }

        // navigation
        "back_to_main_menu" => {
            // interaction
            //     .set_last_interface(ctx.telegram_user_id.to_string().as_str(), "back_to_main_menu")
            //     .await?;
            navigation::handle_callback_back_to_main_menu(&bot, dialogue, message).await?
        }

        // language
        s if s.starts_with("lang:") => {
            let lang_code = s.split(":").nth(1).unwrap_or("en");
            language::handle_callback_language_change(&bot, dialogue, message, lang_code).await?
        }
        _ => todo!(),
    }

    bot.answer_callback_query(&q.id).cache_time(1).await?;

    Ok(())
}

pub fn get_callback_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    Update::filter_callback_query().endpoint(handle_callback)
}
