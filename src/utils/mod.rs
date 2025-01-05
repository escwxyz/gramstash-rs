use error::{BotError, BotResult};
use once_cell::sync::Lazy;
use regex::Regex;
use teloxide::types::Message;
use url::Url;

pub mod error;
pub mod http;
pub mod keyboard;
pub mod redis;

static INSTAGRAM_URL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"https?://(?:www\.)?instagram\.com/[^\s]+").unwrap());

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

// TODO: where to put this?
pub fn block_group_chats(msg: &Message) -> BotResult<()> {
    if msg.chat.id.0 < 0 {
        return Err(BotError::InvalidState("Group chats are not supported".into()));
    }

    Ok(())
}
