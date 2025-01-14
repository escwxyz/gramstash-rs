use teloxide::types::{MessageEntityKind, MessageEntityRef};

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
