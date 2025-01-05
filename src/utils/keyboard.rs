use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

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
    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback("🌐 Language", "language_menu")],
        [InlineKeyboardButton::callback("🔙 Back to Main Menu", "main_menu")],
    ])
}

pub fn get_back_to_menu_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback("🔙 Back to Menu", "main_menu")]])
}
