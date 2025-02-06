use std::sync::LazyLock;

use anyhow::Context;
use regex::Regex;

use super::InstagramError;

static INSTAGRAM_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://(?:www\.)?instagram\.com/[^\s]+")
        .context("Failed to create Instagram URL regex")
        .unwrap()
});

static INSTAGRAM_USERNAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[A-Za-z0-9_][A-Za-z0-9._]*[^.]$|^[A-Za-z0-9]$")
        .context("Failed to create Instagram username regex")
        .unwrap()
});

static INSTAGRAM_PASSWORD_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^.{8,}$")
        .context("Failed to create Instagram password regex")
        .unwrap()
});

pub fn extract_instagram_url(text: &str) -> Option<String> {
    INSTAGRAM_URL_REGEX.find(text).map(|m| m.as_str().to_string())
}

pub fn validate_instagram_username(username: &str) -> bool {
    INSTAGRAM_USERNAME_REGEX.is_match(username)
}

pub fn validate_instagram_password(password: &str) -> bool {
    INSTAGRAM_PASSWORD_REGEX.is_match(password)
}

pub fn normalize_instagram_username(input: &str) -> String {
    // Don't replace double underscores, just handle escaped underscores and trim
    input.replace("\\_", "_").trim().to_string()
}

pub fn process_instagram_username(input: &str) -> Result<String, InstagramError> {
    // First clean the input
    let normalized = normalize_instagram_username(input);
    let cleaned = normalized.trim();

    // Handle common input patterns
    let username = if cleaned.starts_with("@") {
        cleaned.trim_start_matches('@')
    } else if cleaned.starts_with("instagram.com/") {
        cleaned.split("instagram.com/").nth(1).unwrap_or(cleaned)
    } else if cleaned.contains("instagram.com/") {
        cleaned
            .split("instagram.com/")
            .nth(1)
            .unwrap_or(cleaned)
            .split('?')
            .next()
            .unwrap_or(cleaned)
    } else {
        cleaned
    };

    let username = username.trim();

    // Validate the cleaned username
    if username.is_empty() {
        return Err(InstagramError::InvalidUsername("Username cannot be empty".into()));
    }

    if !validate_instagram_username(username) {
        return Err(InstagramError::InvalidUsername(
            "Invalid Instagram username format".into(),
        ));
    }

    Ok(username.to_string())
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

    #[test]
    fn test_normalize_instagram_username() {
        assert_eq!(normalize_instagram_username("__konzentriert"), "__konzentriert"); // Should keep double underscores
        assert_eq!(normalize_instagram_username("\\_\\_konzentriert"), "__konzentriert"); // Should convert escaped underscores
        assert_eq!(normalize_instagram_username("user\\_name"), "user_name"); // Should handle single escaped underscore
        assert_eq!(normalize_instagram_username("  user_name  "), "user_name"); // Should trim spaces
        assert_eq!(normalize_instagram_username("user__name"), "user__name"); // Should keep double underscores
    }
    #[test]
    fn test_process_instagram_username() {
        // Test valid usernames
        assert_eq!(process_instagram_username("user123").unwrap(), "user123");
        assert_eq!(process_instagram_username("@user123").unwrap(), "user123");
        assert_eq!(process_instagram_username("  user123  ").unwrap(), "user123");
        assert_eq!(
            process_instagram_username("https://instagram.com/user123").unwrap(),
            "user123"
        );
        assert_eq!(process_instagram_username("instagram.com/user123").unwrap(), "user123");
        assert_eq!(
            process_instagram_username("https://www.instagram.com/user123?igshid=123").unwrap(),
            "user123"
        );
        assert_eq!(process_instagram_username("__konzentriert").unwrap(), "__konzentriert");
        assert_eq!(process_instagram_username("user____name").unwrap(), "user____name");

        // Test invalid usernames
        assert!(process_instagram_username("").is_err());
        assert!(process_instagram_username(" ").is_err());
        assert!(process_instagram_username(".user123").is_err());
        assert!(process_instagram_username("user123.").is_err());
        assert!(process_instagram_username("user@123").is_err());
        assert!(process_instagram_username("user 123").is_err());
    }
}
