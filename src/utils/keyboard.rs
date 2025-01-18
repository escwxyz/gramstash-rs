// TODO: add navigation buttons

use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

// pub trait MarkupHelper {
//     fn contains_button_with(&self, data: &str) -> bool;
//     fn get_callback_data(&self) -> Option<String>;
// }

// impl MarkupHelper for InlineKeyboardMarkup {
//     fn contains_button_with(&self, data: &str) -> bool {
//         self.inline_keyboard
//             .iter()
//             .flatten()
//             .any(|btn| matches!(&btn.kind, teloxide::types::InlineKeyboardButtonKind::CallbackData(cb_data) if cb_data == data))
//     }

//     fn get_callback_data(&self) -> Option<String> {
//         self.inline_keyboard.iter().flatten().find_map(|btn| {
//             if let teloxide::types::InlineKeyboardButtonKind::CallbackData(cb_data) = &btn.kind {
//                 Some(cb_data.clone())
//             } else {
//                 None
//             }
//         })
//     }
// }

pub struct MainMenu;

impl MainMenu {
    pub fn get_inline_keyboard() -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new([
            [InlineKeyboardButton::callback(
                t!("buttons.main_menu.download"),
                "ask_for_download_link",
            )],
            [InlineKeyboardButton::callback(
                t!("buttons.main_menu.profile"),
                "profile_menu",
            )],
        ])
    }
    pub fn get_back_to_main_menu_keyboard() -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
            t!("buttons.back_to_main_menu"),
            "back_to_main_menu",
        )]])
    }
}

pub struct DownloadMenu;

impl DownloadMenu {
    pub fn get_download_menu_inline_keyboard() -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new([
            [InlineKeyboardButton::callback(
                t!("buttons.download_menu.continue"),
                "ask_for_download_link",
            )],
            [InlineKeyboardButton::callback(
                t!("buttons.download_menu.cancel"),
                "cancel_download",
            )],
        ])
    }

    pub fn get_confirm_download_keyboard() -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new([
            [InlineKeyboardButton::callback(
                t!("buttons.confirm_download.confirm"),
                "confirm_download",
            )],
            [InlineKeyboardButton::callback(
                t!("buttons.confirm_download.cancel"),
                "cancel_download",
            )],
        ])
    }
}

pub struct ProfileMenu;

impl ProfileMenu {
    pub fn get_profile_menu_inline_keyboard(is_authenticated: bool) -> InlineKeyboardMarkup {
        let mut keyboard = Vec::new();

        if is_authenticated {
            keyboard.push(vec![InlineKeyboardButton::callback(
                t!("buttons.profile_menu.logout"),
                "auth_logout",
            )])
        } else {
            keyboard.push(vec![InlineKeyboardButton::callback(
                t!("buttons.profile_menu.login"),
                "auth_login",
            )])
        }

        keyboard.push(vec![InlineKeyboardButton::callback(
            t!("buttons.profile_menu.usage"),
            "show_usage",
        )]);

        keyboard.push(vec![InlineKeyboardButton::callback(
            t!("buttons.back_to_main_menu"),
            "back_to_main_menu",
        )]);

        InlineKeyboardMarkup::new(keyboard)
    }
}

pub struct LoginDialogue;

impl LoginDialogue {
    pub fn get_cancel_auth_keyboard() -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
            t!("buttons.login_dialogue.cancel"),
            "cancel_auth",
        )]])
    }
}

pub struct LogoutMenu;

impl LogoutMenu {
    pub fn get_logout_menu_inline_keyboard() -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new([
            [InlineKeyboardButton::callback(
                t!("buttons.logout_menu.confirm"),
                "confirm_logout",
            )],
            [InlineKeyboardButton::callback(
                t!("buttons.logout_menu.cancel"),
                "cancel_logout",
            )],
        ])
    }
}

pub struct LanguageMenu;

impl LanguageMenu {
    pub fn get_language_menu_inline_keyboard() -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new([
            [InlineKeyboardButton::callback(
                t!("buttons.language_menu.en"),
                "lang:en",
            )],
            [InlineKeyboardButton::callback(
                t!("buttons.language_menu.zh"),
                "lang:zh",
            )],
            [InlineKeyboardButton::callback(
                t!("buttons.language_menu.de"),
                "lang:de",
            )],
            [InlineKeyboardButton::callback(
                t!("buttons.language_menu.fr"),
                "lang:fr",
            )],
            [InlineKeyboardButton::callback(
                t!("buttons.language_menu.ja"),
                "lang:ja",
            )],
            [InlineKeyboardButton::callback(
                t!("buttons.language_menu.es"),
                "lang:es",
            )],
        ])
    }
}
