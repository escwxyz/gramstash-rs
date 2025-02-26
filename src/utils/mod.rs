pub mod http;

#[cfg(test)]
pub mod test;

use teloxide::types::{MessageEntityKind, MessageEntityRef, UserId};

use crate::{config::AppConfig, error::BotError};

/// Reconstructs the original raw text from a message by analyzing its entities.
/// Handles nested formatting by applying inner wrappers first.
/// For example: text with both italic and spoiler will be reconstructed as `||__kon__||`
pub fn reconstruct_raw_text(text: &str, entities: &[MessageEntityRef]) -> String {
    let mut raw_text = String::from(text);

    let mut sorted_entities: Vec<_> = entities.to_vec();

    sorted_entities.sort_by(|a, b| b.range().start.cmp(&a.range().start));

    for entity in sorted_entities.iter() {
        let (prefix, suffix) = match entity.kind() {
            MessageEntityKind::Italic => ("__", "__"),
            MessageEntityKind::Spoiler => ("||", "||"),
            MessageEntityKind::Bold => ("**", "**"),
            MessageEntityKind::Code => ("`", "`"),
            MessageEntityKind::Pre { language: _ } => ("```", "```"),
            MessageEntityKind::Strikethrough => ("~~", "~~"),
            MessageEntityKind::Underline => ("__", "__"),
            _ => continue,
        };

        raw_text.insert_str(entity.range().end, suffix);
        raw_text.insert_str(entity.range().start, prefix);
    }

    raw_text
}

pub fn seconds_to_human_readable(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

pub fn is_admin(user_id: UserId) -> Result<bool, BotError> {
    let config = AppConfig::get()?;
    Ok(config.admin.telegram_user_id == user_id)
}
