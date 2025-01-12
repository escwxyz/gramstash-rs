use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    Image,
    Reel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PostContent {
    Single {
        url: Url,
        content_type: ContentType,
        // caption: Option<String>,
        // timestamp: Option<String>,
    },
    Carousel {
        items: Vec<CarouselItem>,
        // caption: Option<String>,
        // timestamp: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryContent {
    pub url: Url,
    pub content_type: ContentType,
    pub timestamp: Option<String>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaContent {
    Post(PostContent),
    Story(StoryContent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAuthor {
    pub username: String,
    // pub profile_pic_url: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub content: MediaContent,
    pub author: MediaAuthor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarouselItem {
    pub url: Url,
    pub content_type: ContentType,
    // pub caption: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstagramIdentifier {
    Story { username: String, shortcode: String },
    Post { shortcode: String },
    Reel { shortcode: String }, // Note: Reels might need separate handling too
}
