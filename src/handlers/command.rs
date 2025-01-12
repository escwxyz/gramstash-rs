use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::dispatching::{HandlerExt, UpdateHandler};
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::{types::Message, Bot};

use crate::command::Command;
use crate::error::{BotError, HandlerResult};
use crate::services::dialogue::DialogueState;
use crate::state::AppState;
use crate::utils::{is_admin, keyboard};

async fn handle_language(
    bot: Bot,
    _dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    msg: Message,
) -> HandlerResult<()> {
    bot.send_message(msg.chat.id, t!("commands.language"))
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
    let (telegram_user_id, first_name) = match msg.from {
        Some(user) => (user.id.to_string(), user.first_name.clone()),
        // TODO: handle this error
        None => return Err(anyhow::anyhow!("User not found").into()),
    };

    let mut session_service = AppState::get()?.session.lock().await;

    session_service.init_telegram_user_context(&telegram_user_id).await?;

    let welcome_text = t!(
        "commands.start",
        first_name = first_name,
        telegram_user_id = telegram_user_id
    );

    bot.send_message(msg.chat.id, welcome_text)
        .reply_markup(keyboard::MainMenu::get_inline_keyboard()) // TODO!: not showing the inline keyboard
        .reply_markup(keyboard::MainKeyboard::get_keyboard())
        .parse_mode(ParseMode::Html)
        .await?;

    dialogue
        .update(DialogueState::Start)
        .await
        .map_err(|e| BotError::DialogueStateError(e.to_string()))?;

    Ok(())
}

async fn handle_help(bot: Bot, msg: Message) -> HandlerResult<()> {
    bot.send_message(msg.chat.id, t!("commands.help"))
        .reply_markup(keyboard::MainMenu::get_inline_keyboard())
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

async fn handle_unknown_command(bot: Bot, msg: Message) -> HandlerResult<()> {
    bot.send_message(msg.chat.id, t!("commands.unknown_command"))
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    dialogue: Dialogue<DialogueState, ErasedStorage<DialogueState>>,
    state: &AppState,
) -> HandlerResult<()> {
    let user_id = msg
        .clone()
        .from
        // TODO: handle this error
        .ok_or_else(|| BotError::Other(anyhow::anyhow!("User not found")))?
        .id;
    match cmd {
        Command::Start => handle_start(bot, dialogue, msg).await?,
        Command::Help => handle_help(bot, msg).await?,
        Command::Language => handle_language(bot, dialogue, msg).await?,
        Command::Stats | Command::Status if !is_admin(user_id, &state.config.admin)? => {
            handle_unknown_command(bot, msg).await?
        }
        _ => handle_unknown_command(bot, msg).await?,
    }

    Ok(())
}

pub fn get_command_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    Update::filter_message()
        .filter_command::<Command>()
        .endpoint(handle_command)
}

#[cfg(test)]
mod tests {
    use crate::{handlers::get_handler, services::dialogue::DialogueState, utils::test::setup_test_state};
    use teloxide::dptree;
    use teloxide_tests::{MockBot, MockMessageText};

    #[tokio::test]
    async fn test_handle_help() {
        let (test_app_state, storage) = setup_test_state().await.expect("Failed to setup test state");

        let bot = MockBot::new(MockMessageText::new().text("/help"), get_handler());

        bot.dependencies(dptree::deps![storage, test_app_state]);
        bot.set_state(DialogueState::Start).await;

        bot.dispatch().await;

        let responses = bot.get_responses();
        let last_message = responses.sent_messages.last().expect("No messages were sent");

        assert_eq!(last_message.text().expect("Message had no text"), t!("commands.help"));

        assert!(
            last_message.reply_markup().is_some(),
            "Expected reply markup to be present"
        );
    }

    #[tokio::test]
    async fn test_handle_unknown_command() {
        let (test_app_state, storage) = setup_test_state().await.expect("Failed to setup test state");

        let bot = MockBot::new(MockMessageText::new().text("/stats"), get_handler());

        bot.dependencies(dptree::deps![storage, test_app_state]);
        bot.set_state(DialogueState::Start).await;

        bot.dispatch().await;

        let responses = bot.get_responses();
        let last_message = responses.sent_messages.last().expect("No messages were sent");

        assert_eq!(
            last_message.text().expect("Message had no text"),
            t!("commands.unknown_command")
        );
    }
}
