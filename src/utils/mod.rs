pub mod http;
pub mod keyboard;
pub mod redis;

use once_cell::sync::Lazy;
use regex::Regex;
use teloxide::types::UserId;
use url::Url;

use crate::{
    error::{BotError, BotResult},
    state::AppState,
};

static INSTAGRAM_URL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"https?://(?:www\.)?instagram\.com/[^\s]+").unwrap());

static INSTAGRAM_USERNAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Za-z0-9_][A-Za-z0-9._]*[^.]$|^[A-Za-z0-9]$").unwrap());

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instagram_username_regex() {
        // Valid usernames
        assert!(INSTAGRAM_USERNAME_REGEX.is_match("user123"));
        assert!(INSTAGRAM_USERNAME_REGEX.is_match("123user"));
        assert!(INSTAGRAM_USERNAME_REGEX.is_match("123.user"));
        assert!(INSTAGRAM_USERNAME_REGEX.is_match("__konzentriert"));
        assert!(INSTAGRAM_USERNAME_REGEX.is_match("user.name"));
        assert!(INSTAGRAM_USERNAME_REGEX.is_match("user_name"));
        assert!(INSTAGRAM_USERNAME_REGEX.is_match("a")); // Single character
        assert!(INSTAGRAM_USERNAME_REGEX.is_match("user123_._name"));
        assert!(INSTAGRAM_USERNAME_REGEX.is_match("user123._.name"));
        assert!(INSTAGRAM_USERNAME_REGEX.is_match("user_name_")); // Ends with underscore
        assert!(INSTAGRAM_USERNAME_REGEX.is_match("user____name_")); // Ends with underscore
        assert!(INSTAGRAM_USERNAME_REGEX.is_match("_username")); // Starts with underscore
        assert!(INSTAGRAM_USERNAME_REGEX.is_match("username_")); // Ends with underscore
        assert!(INSTAGRAM_USERNAME_REGEX.is_match("username____")); // Ends with underscore

        // Invalid usernames
        assert!(!INSTAGRAM_USERNAME_REGEX.is_match("")); // Empty
        assert!(!INSTAGRAM_USERNAME_REGEX.is_match("user name")); // Space
        assert!(!INSTAGRAM_USERNAME_REGEX.is_match("user@name")); // Special character
        assert!(!INSTAGRAM_USERNAME_REGEX.is_match(".username")); // Starts with dot
        assert!(!INSTAGRAM_USERNAME_REGEX.is_match("username.")); // Ends with dot
    }

    #[test]
    fn test_instagram_url_regex() {
        // Valid URLs
        assert!(INSTAGRAM_URL_REGEX.is_match("https://instagram.com/username"));
        assert!(INSTAGRAM_URL_REGEX.is_match("https://www.instagram.com/username"));
        assert!(INSTAGRAM_URL_REGEX.is_match("http://instagram.com/username"));
        assert!(INSTAGRAM_URL_REGEX.is_match("https://instagram.com/user.name"));
        assert!(INSTAGRAM_URL_REGEX.is_match("https://instagram.com/user_name"));
        assert!(INSTAGRAM_URL_REGEX.is_match("https://instagram.com/p/ABC123"));
        assert!(INSTAGRAM_URL_REGEX.is_match("https://www.instagram.com/reel/ABC123"));

        // Invalid URLs
        assert!(!INSTAGRAM_URL_REGEX.is_match("instagram.com/username")); // Missing protocol
        assert!(!INSTAGRAM_URL_REGEX.is_match("https://instagramm.com/username")); // Wrong domain
        assert!(!INSTAGRAM_URL_REGEX.is_match("https://instagram")); // Incomplete URL
        assert!(!INSTAGRAM_URL_REGEX.is_match("https://instagram.com/")); // Missing username
        assert!(!INSTAGRAM_URL_REGEX.is_match("https://instagram.com/ ")); // Space in URL
    }

    #[test]
    fn test_instagram_password_regex() {
        // Valid passwords
        assert!(INSTAGRAM_PASSWORD_REGEX.is_match("password123"));
        assert!(INSTAGRAM_PASSWORD_REGEX.is_match("12345678"));
        assert!(INSTAGRAM_PASSWORD_REGEX.is_match("abcd1234!@#$"));
        assert!(INSTAGRAM_PASSWORD_REGEX.is_match("verylongpasswordwithspecialchars!@#$"));
        assert!(INSTAGRAM_PASSWORD_REGEX.is_match("        "));

        // Invalid passwords
        assert!(!INSTAGRAM_PASSWORD_REGEX.is_match("")); // Empty
        assert!(!INSTAGRAM_PASSWORD_REGEX.is_match("short")); // Too short
        assert!(!INSTAGRAM_PASSWORD_REGEX.is_match("1234567")); // 7 characters
    }
}
