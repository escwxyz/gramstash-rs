use error::{BotError, BotResult};
use once_cell::sync::Lazy;
use regex::Regex;
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

pub fn escape_markdown(text: &str) -> String {
    const SPECIAL_CHARS: &[char] = &[
        '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!',
    ];

    let mut escaped = String::with_capacity(text.len() * 2);
    for c in text.chars() {
        if SPECIAL_CHARS.contains(&c) {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

pub fn unescape_markdown(text: &str) -> String {
    text.replace('\\', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_markdown() {
        assert_eq!(escape_markdown("hello_world"), "hello\\_world");
        assert_eq!(escape_markdown("*bold*"), "\\*bold\\*");
        assert_eq!(escape_markdown("user.name"), "user\\.name");
        assert_eq!(escape_markdown("(test)"), "\\(test\\)");
        assert_eq!(
            escape_markdown("_*[]()~`>#+-=|{}.!"),
            "\\_\\*\\[\\]\\(\\)\\~\\`\\>\\#\\+\\-\\=\\|\\{\\}\\.\\!"
        );
        assert_eq!(escape_markdown("normal text"), "normal text");
    }

    #[test]
    fn test_unescape_markdown() {
        assert_eq!(unescape_markdown("hello\\_world"), "hello_world");
        assert_eq!(unescape_markdown("\\*bold\\*"), "*bold*");
        assert_eq!(unescape_markdown("user\\.name"), "user.name");
        assert_eq!(unescape_markdown("\\(test\\)"), "(test)");
        assert_eq!(
            unescape_markdown("\\_\\*\\[\\]\\(\\)\\~\\`\\>\\#\\+\\-\\=\\|\\{\\}\\.\\!"),
            "_*[]()~`>#+-=|{}.!"
        );
        assert_eq!(unescape_markdown("normal text"), "normal text");
    }
}
