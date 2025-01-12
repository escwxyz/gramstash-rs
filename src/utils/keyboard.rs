use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup};

pub const DOWNLOAD_BUTTON_EN: &str = "📥 Download";
pub const DOWNLOAD_BUTTON_ZH: &str = "📥 下载";
pub const PROFILE_BUTTON_EN: &str = "👤 Profile";
pub const PROFILE_BUTTON_ZH: &str = "👤 个人资料";

pub struct MainKeyboard;

impl MainKeyboard {
    pub fn get_keyboard() -> KeyboardMarkup {
        KeyboardMarkup::new(vec![
            vec![KeyboardButton::new(t!("buttons.main_menu.download"))],
            vec![KeyboardButton::new(t!("buttons.main_menu.profile"))],
        ])
        .persistent()
        .resize_keyboard()
    }
}

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
    pub fn get_profile_menu_inline_keyboard() -> InlineKeyboardMarkup {
        let mut keyboard = Vec::new();
        // TODO! user status

        // let state = AppState::get()?;

        // let instagram_service = state.instagram.lock().await?;

        keyboard.push(vec![
            InlineKeyboardButton::callback(t!("buttons.profile_menu.login"), "auth_login"),
            InlineKeyboardButton::callback(t!("buttons.profile_menu.usage"), "show_usage"),
        ]);

        keyboard.push(vec![InlineKeyboardButton::callback("❌ Cancel", "cancel")]);

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

pub struct LanguageMenu;

impl LanguageMenu {
    pub fn get_language_menu_inline_keyboard() -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new([
            [InlineKeyboardButton::callback(
                t!("buttons.language_menu.en"),
                "language_en",
            )],
            [InlineKeyboardButton::callback(
                t!("buttons.language_menu.zh"),
                "language_zh",
            )],
        ])
    }
}
