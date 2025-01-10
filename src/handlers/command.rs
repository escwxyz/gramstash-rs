use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::dispatching::{HandlerExt, UpdateHandler};
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::{types::Message, Bot};

use crate::command::Command;
use crate::services::dialogue::DialogueState;
use crate::state::AppState;
use crate::utils::error::{BotError, HandlerResult};
use crate::utils::{is_admin, keyboard};

async fn handle_language(
    bot: Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: Message,
) -> HandlerResult<()> {
    let language_text = t!("commands.language").to_string();

    let msg = bot
        .send_message(msg.chat.id, language_text)
        .reply_markup(keyboard::LanguageMenu::get_language_menu_inline_keyboard())
        .parse_mode(ParseMode::Html)
        .await?;

    // TODO: update session with language

    // bot.edit_message

    // dialogue
    //     .update(DialogueState::AwaitingLanguage {
    //         prompt_msg_id: msg.id,
    //         language: "en".to_string(),
    //     })
    //     .await?;

    Ok(())
}

async fn handle_start(
    bot: Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: Message,
) -> HandlerResult<()> {
    // Just check what is the user's language

    info!("Checking user ... {:?}", msg.from);

    let (telegram_user_id, first_name) = match msg.from {
        Some(user) => (user.id.to_string(), user.first_name.clone()),
        None => return Err(anyhow::anyhow!("User not found").into()),
    };

    let mut session_service = AppState::get()?.session.lock().await;

    session_service.init_telegram_user_context(&telegram_user_id).await?;

    let welcome_text = t!(
        "commands.start",
        first_name = first_name,
        telegram_user_id = telegram_user_id
    )
    .to_string();

    bot.send_message(msg.chat.id, welcome_text)
        .reply_markup(keyboard::MainMenu::get_inline_keyboard())
        .reply_markup(keyboard::MainKeyboard::get_keyboard())
        .parse_mode(ParseMode::Html)
        .await?;

    dialogue.update(DialogueState::Start).await?;

    Ok(())
}

async fn handle_help(bot: Bot, msg: Message) -> HandlerResult<()> {
    let help_text = t!("commands.help").to_string();

    bot.send_message(msg.chat.id, help_text)
        .reply_markup(keyboard::MainMenu::get_inline_keyboard())
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

async fn handle_unknown_command(bot: Bot, msg: Message) -> HandlerResult<()> {
    bot.send_message(
        msg.chat.id,
        "⚠️ Unknown command\n\nPlease use /help to see available commands.",
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
) -> HandlerResult<()> {
    let user_id = msg
        .clone()
        .from
        .ok_or_else(|| BotError::Other(anyhow::anyhow!("User not found")))?
        .id;
    match cmd {
        Command::Start => handle_start(bot, dialogue, msg).await?,
        Command::Help => handle_help(bot, msg).await?,
        Command::Language => handle_language(bot, dialogue, msg).await?,
        Command::Stats | Command::Status if !is_admin(user_id)? => handle_unknown_command(bot, msg).await?,
        _ => handle_unknown_command(bot, msg).await?,
    }

    Ok(())
}

pub fn get_command_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    Update::filter_message()
        .filter_command::<Command>()
        .endpoint(handle_command)
}
