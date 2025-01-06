use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::services::instagram::InstagramService;

pub fn get_main_menu_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback("📥 Download Content", "download_menu")],
        [InlineKeyboardButton::callback("⚙️ Settings", "settings_menu")],
        [InlineKeyboardButton::callback("ℹ️ Help", "help_menu")],
    ])
}

pub fn get_download_menu_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback("📸 Post/Reel", "download_post")],
        [InlineKeyboardButton::callback("📖 Story", "download_story")],
        [InlineKeyboardButton::callback("🔙 Back to Main Menu", "main_menu")],
    ])
}

pub fn get_confirm_download_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[
        InlineKeyboardButton::callback("✅ Confirm", "confirm"),
        InlineKeyboardButton::callback("❌ Cancel", "cancel"),
    ]])
}

pub fn get_settings_keyboard() -> InlineKeyboardMarkup {
    let instagram_service = InstagramService::new();
    let username = instagram_service.get_username();

    let mut buttons = vec![[InlineKeyboardButton::callback("🌐 Language", "language_menu")]];

    if username.is_none() {
        buttons.push([InlineKeyboardButton::callback("🔑 Login", "login")]);
    } else {
        buttons.push([InlineKeyboardButton::callback(
            format!("🔓 Logout {}", username.unwrap_or_default()),
            "logout",
        )]);
    }

    buttons.push([InlineKeyboardButton::callback("🔓 Back to Main Menu", "main_menu")]);

    InlineKeyboardMarkup::new(buttons)
}

pub fn get_back_to_menu_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback("🔙 Back to Menu", "main_menu")]])
}
