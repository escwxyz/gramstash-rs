use super::{instagram::extract_instagram_url, Platform};

pub fn extract_url_from_message(platform: &Platform, text: &str) -> Option<String> {
    match platform {
        Platform::Instagram => extract_instagram_url(text),
        Platform::Youtube => todo!(),
        Platform::Bilibili => todo!(),
    }
}
