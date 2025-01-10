use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::dispatching::{HandlerExt, UpdateHandler};
use teloxide::prelude::*;
use teloxide::{types::Message, Bot};

use crate::command::Command;
use crate::services::dialogue::DialogueState;
use crate::state::AppState;
use crate::utils::error::{BotError, HandlerResult};
use crate::utils::{is_admin, keyboard};

async fn handle_language(bot: Bot, msg: Message) -> HandlerResult<()> {
    let _msg = bot
        .send_message(msg.chat.id, "🌐 Please select a language below 👇")
        .reply_markup(keyboard::LanguageMenu::get_language_menu_inline_keyboard())
        .await?;

    // TODO: update session with language

    // bot.edit_message

    Ok(())
}

async fn handle_start(
    bot: Bot,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: Message,
) -> HandlerResult<()> {
    let (telegram_user_id, user_name) = match msg.from {
        Some(user) => (user.id.to_string(), user.first_name.clone()),
        None => return Err(anyhow::anyhow!("User not found").into()),
    };

    // Check if user has existing session
    // let mut session_service = AppState::get()?.session.lock().await;
    // let is_returning_user = session_service.validate_session(&telegram_user_id).await?;

    let mut session_service = AppState::get()?.session.lock().await;

    session_service.init_telegram_user_context(&telegram_user_id).await?;

    // let welcome_text = if is_returning_user {
    //     format!(
    //         "👋 Welcome back <b>{}</b>!\n\n\
    //         I'm ready to help you download content from Instagram.\n\n\
    //         Please select an option below:",
    //         user_name
    //     )
    // } else {
    //     format!(
    //         "👋 Hi <b>{}</b>!\n\n\
    //         Welcome to GramStash! I can help you download content from Instagram.\n\n\
    //         Please select an option below:",
    //         user_name
    //     )
    // };

    let welcome_text = format!(
        "👋 Hi {}!\n\n\
        Welcome to GramStash! I can help you download content from Instagram.\n\n\
        Please select an option below:",
        user_name
    );

    bot.send_message(msg.chat.id, welcome_text)
        .reply_markup(keyboard::MainKeyboard::get_keyboard())
        .reply_markup(keyboard::MainMenu::get_inline_keyboard())
        .await?;

    dialogue.update(DialogueState::Start).await?;

    Ok(())
}

async fn handle_help(bot: Bot, msg: Message) -> HandlerResult<()> {
    // TODO: make more comprehensive
    let help_text = "This is a help message\n\n
    You can download content from Instagram by selecting the download option from the main menu.\n\n
    You can also manage your profile and subscription by selecting the profile option from the main menu.\n\n
    For faster access, you can simply click the persistent buttons in your keyboard.\n\n
    Got any questions? Use /language to change the language of the bot.\n\n
    ";

    bot.send_message(msg.chat.id, help_text)
        .reply_markup(keyboard::MainMenu::get_inline_keyboard())
        .await?;

    Ok(())
}

async fn handle_unknown_command(bot: Bot, msg: Message) -> HandlerResult<()> {
    bot.send_message(
        msg.chat.id,
        "⚠️ Unknown command\n\nPlease use /help to see available commands.",
    )
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
        Command::Language => handle_language(bot, msg).await?,
        Command::Stats | Command::Status if is_admin(user_id)? => handle_unknown_command(bot, msg).await?,
        _ => handle_unknown_command(bot, msg).await?,
    }

    Ok(())
}

pub fn get_command_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    Update::filter_message()
        .filter_command::<Command>()
        .endpoint(handle_command)
}
