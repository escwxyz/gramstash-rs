use error::{BotError, BotResult};
use once_cell::sync::Lazy;
use regex::Regex;
use teloxide::types::UserId;
use url::Url;

use crate::state::AppState;

pub mod error;
pub mod http;
pub mod keyboard;
pub mod redis;

static INSTAGRAM_URL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"https?://(?:www\.)?instagram\.com/[^\s]+").unwrap());

static INSTAGRAM_USERNAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?!.*\.\.)(?!.*\.$)[^\W][\w.]{0,29}$").unwrap());

static INSTAGRAM_PASSWORD_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^.{8,}$").unwrap());

pub fn parse_url(url: &str) -> BotResult<Url> {
    let parsed_url = Url::parse(url).map_err(|_| BotError::InvalidUrl("Invalid URL format".into()))?;

    if parsed_url.host_str() == Some("instagram.com") || parsed_url.host_str() == Some("www.instagram.com") {
        Ok(parsed_url)
    } else {
        Err(BotError::InvalidUrl("Not an Instagram URL".into()))
    }
}

pub fn extract_instagram_url(text: &str) -> Option<String> {
    INSTAGRAM_URL_REGEX.find(text).map(|m| m.as_str().to_string())
}

pub fn validate_instagram_username(username: &str) -> bool {
    INSTAGRAM_USERNAME_REGEX.is_match(username)
}

pub fn validate_instagram_password(password: &str) -> bool {
    INSTAGRAM_PASSWORD_REGEX.is_match(password)
}

pub fn is_admin(user_id: UserId) -> BotResult<bool> {
    let admin_config = AppState::get()?.config.admin.clone();
    Ok(admin_config.telegram_user_id == user_id)
}
