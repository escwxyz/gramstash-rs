use teloxide::types::Update;

use crate::{
    error::{BotError, BotResult, MiddlewareError},
    utils::validate_instagram_username,
};

pub fn extract_user_id(update: &Update) -> Option<String> {
    update.from().map(|user| user.id.to_string())
}

pub fn normalize_instagram_username(input: &str) -> String {
    // Don't replace double underscores, just handle escaped underscores and trim
    input.replace("\\_", "_").trim().to_string()
}

pub fn process_instagram_username(input: &str) -> BotResult<String> {
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
        return Err(BotError::ServiceError(crate::error::ServiceError::Middleware(
            MiddlewareError::ValidationError("Username cannot be empty".into()),
        )));
    }

    if !validate_instagram_username(username) {
        return Err(BotError::ServiceError(crate::error::ServiceError::Middleware(
            MiddlewareError::ValidationError("Invalid Instagram username format".into()),
        )));
    }

    Ok(username.to_string())
}

// fn requires_auth(dialogue_state: &DialogueState) -> bool {
//     match dialogue_state {
//         DialogueState::AwaitingDownloadLink(_)
//         | DialogueState::AwaitingLogoutConfirmation(_)
//         | DialogueState::ConfirmLogout => true,

//         DialogueState::ConfirmDownload { content } => match content {
//             MediaContent::Post(_) => false,
//             MediaContent::Story(_) => true,
//         },

//         _ => false,
//     }
// }

// pub async fn handle_user_session(telegram_user_id: &str, dialogue_state: &DialogueState) -> BotResult<()> {
//     let state = AppState::get()?;
//     let session_service = state.session.lock().await;

//     // Only validate session for states that require authentication
//     if requires_auth(dialogue_state) {
//         if !session_service.validate_session(telegram_user_id).await? {
//             return Err(BotError::Other(anyhow::anyhow!("Invalid or expired session")));
//         }

//         // Restore Instagram client cookies if needed
//         if let Some(stored_session) = session_service.get_session(telegram_user_id).await? {
//             if let Some(session_data) = stored_session.session_data {
//                 let mut instagram_service = state.instagram.lock().await;
//                 instagram_service.restore_cookies(session_data)?;
//             }
//         }
//     }

//     Ok(())
// }

#[cfg(test)]
mod tests {
    use super::*;

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
