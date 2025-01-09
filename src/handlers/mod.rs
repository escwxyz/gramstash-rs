mod callback;
mod command;
mod message;

pub use command::Command;

use callback::get_callback_handler;
use command::get_command_handler;
use message::get_message_handler;
use teloxide::{
    dispatching::{dialogue::ErasedStorage, HandlerExt, UpdateFilterExt, UpdateHandler},
    dptree,
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{Message, Update},
    Bot,
};

use crate::{services::dialogue::DialogueState, utils::keyboard};

pub fn get_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    Update::filter_message()
        .enter_dialogue::<Message, ErasedStorage<DialogueState>, DialogueState>()
        .branch(get_command_handler())
        .branch(get_message_handler())
        .branch(get_callback_handler())
        .branch(dptree::endpoint(|msg: Message, bot: Bot| async move {
            bot.delete_message(msg.chat.id, msg.id).await?;
            bot.send_message(
                msg.chat.id,
                "🤷‍♂️ Unknown message.\n\nPlease click the following keyboard buttons to continue.\n\n",
            )
            .reply_markup(keyboard::MainMenu::get_inline_keyboard())
            .await?;

            Ok(())
        }))
}
