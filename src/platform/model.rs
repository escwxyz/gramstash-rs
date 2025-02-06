use std::{
    fmt::{self, Display},
    str::FromStr,
};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::service::Cacheable;

use super::{instagram::model::InstagramIdentifier, PlatformError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
pub enum MediaType {
    Image,
    Video,
    Audio,
}

impl Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlatformIdentifier {
    Instagram(InstagramIdentifier),
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Platform {
    #[default]
    Instagram,
    Youtube,
    Bilibili,
}

impl ToString for Platform {
    fn to_string(&self) -> String {
        match self {
            Self::Instagram => "Instagram".to_string(),
            Self::Youtube => "Youtube".to_string(),
            Self::Bilibili => "Bilibili".to_string(),
        }
    }
}

impl FromStr for Platform {
    type Err = PlatformError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "instagram" => Ok(Self::Instagram),
            "youtube" => Ok(Self::Youtube),
            "bilibili" => Ok(Self::Bilibili),
            _ => Err(PlatformError::InvalidPlatform(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub enum MediaContentType {
    Single,
    Multiple,
    Story,
    Playlist,
    Album,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct MediaAuthor {
    pub id: String,
    pub username: String,
}

// ------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct MediaFile {
    pub id: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<MediaAuthor>,
    pub content_type: MediaContentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<Url>,
    pub items: Vec<MediaFileItem>,
    pub platform: Platform,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct MediaFileItem {
    pub id: String,
    pub media_type: MediaType,
    pub url: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<Duration>,
    pub created_at: DateTime<Utc>,
}

impl Cacheable for MediaFile {
    fn cache_prefix() -> &'static str {
        "media_file"
    }

    fn cache_key(&self) -> String {
        format!("{}:{}", self.platform.to_string().to_lowercase(), self.id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadState {
    RateLimited,
    Success(MediaFile),
    Error, // TODO: add error inside
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PostDownloadState {
    Success,
    Error,
}
