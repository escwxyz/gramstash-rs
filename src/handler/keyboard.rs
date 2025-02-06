use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::{error::BotResult, platform::Platform, state::AppState};

pub async fn get_platform_keyboard() -> BotResult<InlineKeyboardMarkup> {
    let app_state = AppState::get()?;
    let supported_platforms = app_state.platform_registry.get_supported_platforms().await;

    let mut buttons = Vec::new();
    for platform in supported_platforms {
        let text = format!("buttons.platforms.{}", platform.to_string().to_lowercase());

        let callback_data = format!("platform:{}", platform.to_string().to_lowercase());

        buttons.push(vec![InlineKeyboardButton::callback(t!(text), callback_data)]);
    }

    buttons.push(vec![InlineKeyboardButton::callback(
        t!("buttons.back_to_main_menu"),
        "back_to_main_menu",
    )]);

    Ok(InlineKeyboardMarkup::new(buttons))
}

pub fn get_main_menu_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback(
            t!("buttons.main_menu.download"),
            "select_platform_menu",
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

pub fn get_download_ask_for_link_keyboard(platform: Platform) -> InlineKeyboardMarkup {
    let callback_data = format!("platform:{}", platform.to_string().to_lowercase());

    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback(
            t!("buttons.download_menu.continue"),
            callback_data,
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

pub fn get_profile_menu_keyboard() -> InlineKeyboardMarkup {
    let mut keyboard = Vec::new();

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

pub fn get_cancel_auth_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
        t!("buttons.login_dialogue.cancel"),
        "cancel_auth",
    )]])
}

pub fn get_language_menu_keyboard() -> InlineKeyboardMarkup {
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
