pub enum Language {
    English,
    Chinese,
}

impl Language {
    pub fn get_locale(&self) -> &str {
        match self {
            Language::English => "en",
            Language::Chinese => "zh",
        }
    }

    // pub fn get_translation(&self, dialogue_state: DialogueState, key: &str) -> String {
    //     rust_i18n::set_locale(self.get_locale());

    //     let text = match dialogue_state {
    //         DialogueState::Start => todo!(),
    //         DialogueState::AwaitingDownloadLink(message_id) => todo!(),
    //         DialogueState::ConfirmDownload { content } => todo!(),
    //         DialogueState::AwaitingUsername(message_id) => todo!(),
    //         DialogueState::AwaitingPassword {
    //             username,
    //             prompt_msg_id,
    //         } => todo!(),
    //         DialogueState::AwaitingLogoutConfirmation(message_id) => todo!(),
    //         DialogueState::ConfirmLogout => todo!(),
    //     };
    // }
}
