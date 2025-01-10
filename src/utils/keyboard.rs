use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup};

pub const DOWNLOAD_BUTTON: &str = "📥 Download";
pub const PROFILE_BUTTON: &str = "👤 Profile";

pub struct MainKeyboard;

impl MainKeyboard {
    pub fn get_keyboard() -> KeyboardMarkup {
        KeyboardMarkup::new(vec![
            vec![KeyboardButton::new(DOWNLOAD_BUTTON)],
            vec![KeyboardButton::new(PROFILE_BUTTON)],
        ])
        .persistent()
        .resize_keyboard()
    }
}

pub struct MainMenu;

impl MainMenu {
    pub fn get_inline_keyboard() -> InlineKeyboardMarkup {
        let download_text = t!("buttons.main_menu.download").to_string();
        let profile_text = t!("buttons.main_menu.profile").to_string();

        InlineKeyboardMarkup::new([
            [InlineKeyboardButton::callback(download_text, "ask_for_download_link")],
            [InlineKeyboardButton::callback(profile_text, "profile_menu")],
        ])
    }
}

pub struct DownloadMenu;

impl DownloadMenu {
    pub fn get_download_menu_inline_keyboard() -> InlineKeyboardMarkup {
        let download_text = t!("buttons.download_menu").to_string();
        let cancel_text = t!("buttons.download_menu").to_string();

        InlineKeyboardMarkup::new([
            [InlineKeyboardButton::callback(download_text, "ask_for_download_link")],
            [InlineKeyboardButton::callback(cancel_text, "cancel_download")],
        ])
    }

    pub fn get_confirm_download_keyboard() -> InlineKeyboardMarkup {
        let confirm_text = t!("buttons.confirm_download.confirm").to_string();
        let cancel_text = t!("buttons.confirm_download.cancel").to_string();

        InlineKeyboardMarkup::new([
            [InlineKeyboardButton::callback(confirm_text, "confirm_download")],
            [InlineKeyboardButton::callback(cancel_text, "cancel_download")],
        ])
    }
}

pub struct ProfileMenu;

impl ProfileMenu {
    pub fn get_profile_menu_inline_keyboard() -> InlineKeyboardMarkup {
        let mut keyboard = Vec::new();
        // todo user status
        keyboard.push(vec![
            InlineKeyboardButton::callback("🔑 Login", "auth_login"),
            InlineKeyboardButton::callback("📊 Usage", "show_usage"),
        ]);

        keyboard.push(vec![InlineKeyboardButton::callback("❌ Cancel", "cancel")]);

        InlineKeyboardMarkup::new(keyboard)
    }
}

pub struct LoginDialogue;

impl LoginDialogue {
    pub fn get_cancel_auth_keyboard() -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new([[InlineKeyboardButton::callback("🔙 Back", "cancel_auth")]])
    }
}

pub struct LanguageMenu;

impl LanguageMenu {
    pub fn get_language_menu_inline_keyboard() -> InlineKeyboardMarkup {
        let language_en = t!("buttons.language_menu.en").to_string();
        let language_zh = t!("buttons.language_menu.zh").to_string();

        InlineKeyboardMarkup::new([
            [InlineKeyboardButton::callback(language_en, "language_en")],
            [InlineKeyboardButton::callback(language_zh, "language_zh")],
        ])
    }
}
