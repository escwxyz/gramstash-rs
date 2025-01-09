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
        InlineKeyboardMarkup::new([
            [InlineKeyboardButton::callback("📥 Download", "ask_for_download_link")],
            [InlineKeyboardButton::callback("👤 Profile", "profile_menu")],
        ])
    }
}

pub struct DownloadMenu;

impl DownloadMenu {
    pub fn get_download_menu_inline_keyboard() -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new([
            [InlineKeyboardButton::callback(
                "🔍 Continue Download",
                "ask_for_download_link",
            )],
            [InlineKeyboardButton::callback("❌ Cancel Download", "cancel_download")],
        ])
    }

    pub fn get_confirm_download_keyboard() -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new([
            [InlineKeyboardButton::callback("✅ Confirm", "confirm_download")],
            [InlineKeyboardButton::callback("❌ Cancel", "cancel_download")],
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

pub struct LanguageMenu;

impl LanguageMenu {
    pub fn get_inline_keyboard() -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new([
            [InlineKeyboardButton::callback("🇺🇸 English", "language_en")],
            [InlineKeyboardButton::callback("🇨🇳 Chinese", "language_cn")],
        ])
    }
}
