use teloxide::types::Update;

pub fn check_private_chat(update: &Update) -> bool {
    if let Some(chat) = update.chat() {
        if chat.is_private() {
            return true;
        }
    }
    false
}
