use teloxide::{
    dispatching::dialogue::ErasedStorage,
    payloads::EditMessageTextSetters,
    prelude::{Dialogue, Requester},
    types::{MaybeInaccessibleMessage, ParseMode},
    Bot,
};

use crate::{
    services::{dialogue::DialogueState, language::Language},
    state::AppState,
    utils::error::HandlerResult,
};

pub(super) async fn handle_callback_language_en(
    bot: &Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    message: MaybeInaccessibleMessage,
) -> HandlerResult<()> {
    info!("handle_callback_language_en");
    // Store current state before updating language
    let current_state = dialogue.get().await?.unwrap_or(DialogueState::Start);

    let mut language = AppState::get()?.language.lock().await;
    *language = Language::English;
    rust_i18n::set_locale(language.get_locale());

    let new_text = get_text_for_state(&current_state)?;

    bot.edit_message_text(message.chat().id, message.id(), new_text).await?;

    // Restore the same state
    dialogue.update(current_state).await?;

    Ok(())
}

pub(super) async fn handle_callback_language_zh(
    bot: &Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    message: MaybeInaccessibleMessage,
) -> HandlerResult<()> {
    info!("handle_callback_language_zh");
    // Store current state before updating language
    let current_state = dialogue.get().await?.unwrap_or(DialogueState::Start);

    let mut language = AppState::get()?.language.lock().await;
    *language = Language::Chinese;
    rust_i18n::set_locale(language.get_locale());

    // Get the appropriate text based on current state
    let new_text = get_text_for_state(&current_state)?;

    bot.edit_message_text(message.chat().id, message.id(), new_text)
        .parse_mode(ParseMode::Html)
        .await?;

    // Restore the same state
    dialogue.update(current_state).await?;

    Ok(())
}

// Helper function to get the appropriate text based on dialogue state
fn get_text_for_state(state: &DialogueState) -> HandlerResult<String> {
    let text = match state {
        DialogueState::Start => t!("commands.start"),
        // Add other states here with their corresponding translations
        _ => t!("commands.start"), // fallback to start text
    };
    Ok(text.to_string())
}
