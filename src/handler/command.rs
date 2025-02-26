use teloxide::adaptors::Throttle;
use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::dispatching::{HandlerExt, UpdateHandler};
use teloxide::prelude::*;
use teloxide::{types::Message, Bot};

use crate::command::{self, Command};
use crate::context::UserContext;
use crate::error::{BotError, HandlerResult};
use crate::service::dialogue::model::DialogueState;
use crate::utils::is_admin;

use super::keyboard::{get_language_menu_keyboard, get_main_menu_keyboard};

async fn handle_language(bot: Throttle<Bot>, msg: Message) -> HandlerResult<()> {
    bot.delete_message(msg.chat.id, msg.id).await?;
    bot.send_message(msg.chat.id, t!("commands.language"))
        .reply_markup(get_language_menu_keyboard())
        .await?;

    Ok(())
}

async fn handle_start(
    bot: Throttle<Bot>,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: Message,
) -> HandlerResult<()> {
    let user_id = msg.from.clone().unwrap().id;

    let first_name = msg.from.clone().unwrap().first_name;

    let is_admin = is_admin(user_id)?;

    UserContext::ensure_initialized(user_id, first_name, is_admin).await;

    let context = UserContext::global();

    info!("context: {:?}", context);

    let welcome_text = t!(
        "commands.start.unauthenticated",
        first_name = context.user_name(),
        telegram_user_id = context.user_id().to_string()
    );

    bot.delete_message(msg.chat.id, msg.id).await?;

    bot.send_message(msg.chat.id, welcome_text)
        .reply_markup(get_main_menu_keyboard())
        .await?;

    dialogue
        .update(DialogueState::Start)
        .await
        .map_err(|e| BotError::DialogueStateError(e.to_string()))?;

    // setup commands
    if is_admin {
        command::setup_admin_commands(&bot, msg.chat.id).await?;
    } else {
        command::setup_user_commands(&bot).await?;
    }

    Ok(())
}

async fn handle_help(bot: Throttle<Bot>, msg: Message) -> HandlerResult<()> {
    bot.delete_message(msg.chat.id, msg.id).await?;
    bot.send_message(msg.chat.id, t!("commands.help"))
        .reply_markup(get_main_menu_keyboard())
        .await?;

    Ok(())
}

async fn handle_unknown_command(bot: Throttle<Bot>, msg: Message) -> HandlerResult<()> {
    bot.delete_message(msg.chat.id, msg.id).await?;
    bot.send_message(msg.chat.id, t!("commands.unknown_command")).await?;
    Ok(())
}

// async fn handle_stats(bot: Throttle<Bot>, msg: Message) -> HandlerResult<()> {
//     bot.delete_message(msg.chat.id, msg.id).await?;

//     let processing_msg = bot.send_message(msg.chat.id, t!("commands.stats.processing")).await?;

//     // let total_users = cache_service.keys(pattern)

//     // get stats from db
//     // stats include
//     // - total number of users
//     // TODO
//     bot.edit_message_text(msg.chat.id, processing_msg.id, t!("commands.stats"))
//         .await?;

//     Ok(())
// }

async fn handle_command(
    bot: Throttle<Bot>,
    msg: Message,
    cmd: Command,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
) -> HandlerResult<()> {
    match cmd {
        Command::Start => handle_start(bot, dialogue, msg).await?,
        Command::Help => handle_help(bot, msg).await?,
        Command::Language => handle_language(bot, msg).await?,
        // Command::Stats if is_admin(msg.clone().from.unwrap().id)? => handle_stats(bot, msg).await?,
        _ => handle_unknown_command(bot, msg).await?,
    }

    Ok(())
}

pub fn get_command_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    Update::filter_message()
        .filter_command::<Command>()
        .endpoint(handle_command)
}
