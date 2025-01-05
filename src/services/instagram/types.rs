use serde::{Deserialize, Serialize};
use url::Url;

// ------------------------------

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub message: String,
    pub status: String,
    #[serde(default)]
    pub authenticated: bool,
    #[serde(default)]
    pub user: bool,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub two_factor_required: Option<bool>,
    #[serde(default)]
    pub two_factor_info: Option<TwoFactorInfo>,
    #[serde(default)]
    pub checkpoint_url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TwoFactorInfo {
    pub two_factor_identifier: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionData {
    pub cookies: Vec<SerializableCookie>,
    pub user_id: Option<String>,    // ds_user_id
    pub username: Option<String>,   // we keep this for convenience
    pub csrf_token: Option<String>, // csrftoken
    pub session_id: Option<String>, // sessionid
    pub device_id: Option<String>,  // ig_did
    pub machine_id: Option<String>, // mid
    pub rur: Option<String>,        // rur
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializableCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
}

#[derive(Clone)]
pub struct TwoFactorAuthPending {
    pub user: String,
    pub two_factor_identifier: String,
}

// ------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
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
