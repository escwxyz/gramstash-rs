use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::platform::{
    traits::IntoMediaInfo, MediaAuthor, MediaContentType, MediaInfo, MediaItem, MediaType, Platform, PlatformError,
};

use super::InstagramError;

#[derive(Debug, Clone, PartialEq)]
pub enum InstagramIdentifier {
    Story { username: String, story_id: String },
    Post { shortcode: String },
    Reel { shortcode: String },
}

// --- ---

impl IntoMediaInfo for InstagramMedia {
    fn into_media_info(self) -> Result<MediaInfo, PlatformError> {
        Ok(self.try_into()?)
    }
}

// InstagramMedia <=> MediaInfo
impl TryFrom<InstagramMedia> for MediaInfo {
    type Error = InstagramError;

    fn try_from(media: InstagramMedia) -> Result<Self, Self::Error> {
        let (content_type, items) = match media.content {
            InstagramContent::Single(item) => {
                let media_item = MediaItem {
                    id: item.id,
                    media_type: item.media_type,
                    url: Url::parse(&item.url).map_err(|e| InstagramError::InvalidUrl(e.to_string()))?,
                    thumbnail: Url::parse(&item.thumbnail_url)
                        .map_err(|e| InstagramError::InvalidUrl(e.to_string()))?,
                    duration: None,
                    created_at: item.timestamp,
                };
                (MediaContentType::Single, vec![media_item])
            }
            InstagramContent::Multiple(items) => {
                let media_items = items
                    .into_iter()
                    .map(|item| {
                        Ok(MediaItem {
                            id: item.id,
                            media_type: item.media_type,
                            url: Url::parse(&item.url).map_err(|e| InstagramError::InvalidUrl(e.to_string()))?,
                            thumbnail: Url::parse(&item.thumbnail_url)
                                .map_err(|e| InstagramError::InvalidUrl(e.to_string()))?,
                            duration: None,
                            created_at: item.timestamp,
                        })
                    })
                    .collect::<Result<Vec<_>, InstagramError>>()?;
                (MediaContentType::Multiple, media_items)
            }
            InstagramContent::Story(item) => {
                let media_item = MediaItem {
                    id: item.id,
                    media_type: item.media_type,
                    url: Url::parse(&item.url).map_err(|e| InstagramError::InvalidUrl(e.to_string()))?,
                    thumbnail: Url::parse(&item.thumbnail_url)
                        .map_err(|e| InstagramError::InvalidUrl(e.to_string()))?,
                    duration: None,
                    created_at: item.timestamp,
                };
                (MediaContentType::Story, vec![media_item])
            }
        };

        Ok(MediaInfo {
            identifier: media.shortcode,
            created_at: media.timestamp,
            title: None,
            description: media.caption,
            author: Some(MediaAuthor {
                id: media.author.id,
                username: media.author.username,
            }),
            content_type,
            items,
            platform: Platform::Instagram,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstagramMedia {
    pub id: String,
    pub shortcode: String,
    pub author: InstagramAuthor,
    pub caption: Option<String>,
    pub content: InstagramContent,
    pub thumbnail_url: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstagramAuthor {
    pub id: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstagramContent {
    Single(InstagramMediaItem),
    Multiple(Vec<InstagramMediaItem>),
    Story(InstagramMediaItem),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstagramMediaItem {
    pub id: String,
    pub media_type: MediaType,
    pub url: String,
    pub thumbnail_url: String,
    pub timestamp: DateTime<Utc>,
}

impl TryFrom<XDTGraphImage> for InstagramMedia {
    type Error = InstagramError;

    fn try_from(image: XDTGraphImage) -> Result<Self, Self::Error> {
        Ok(InstagramMedia {
            id: image.id.clone(),
            shortcode: image.shortcode,
            author: InstagramAuthor {
                id: image.owner.id,
                username: image.owner.username,
            },
            thumbnail_url: image.display_url.clone(),
            timestamp: DateTime::from_timestamp(image.taken_at_timestamp, 0)
                .context("Failed to parse timestamp")
                .unwrap(),
            caption: image
                .edge_media_to_caption
                .edges
                .first()
                .map(|edge| edge.node.text.clone()),
            content: InstagramContent::Single(InstagramMediaItem {
                id: image.id.clone(),
                media_type: MediaType::Image,
                url: image.display_url.clone(),
                thumbnail_url: image.display_url.clone(),
                timestamp: DateTime::from_timestamp(image.taken_at_timestamp, 0)
                    .context("Failed to parse timestamp")
                    .unwrap(),
            }),
        })
    }
}

impl TryFrom<XDTGraphVideo> for InstagramMedia {
    type Error = InstagramError;

    fn try_from(video: XDTGraphVideo) -> Result<Self, Self::Error> {
        Ok(InstagramMedia {
            id: video.id.clone(),
            shortcode: video.shortcode,
            author: InstagramAuthor {
                id: video.owner.id,
                username: video.owner.username,
            },
            timestamp: DateTime::from_timestamp(video.taken_at_timestamp, 0)
                .context("Failed to parse timestamp")
                .unwrap(),
            thumbnail_url: video.display_url.clone(),
            caption: video
                .edge_media_to_caption
                .edges
                .first()
                .map(|edge| edge.node.text.clone()),
            content: InstagramContent::Single(InstagramMediaItem {
                id: video.id.clone(),
                media_type: MediaType::Video,
                url: video.video_url.clone(),
                thumbnail_url: video.display_url.clone(),
                timestamp: DateTime::from_timestamp(video.taken_at_timestamp, 0)
                    .context("Failed to parse timestamp")
                    .unwrap(),
            }),
        })
    }
}

impl TryFrom<XDTGraphSidecar> for InstagramMedia {
    type Error = InstagramError;

    fn try_from(sidecar: XDTGraphSidecar) -> Result<Self, Self::Error> {
        let items = sidecar
            .edge_sidecar_to_children
            .edges
            .into_iter()
            .filter_map(|edge| match edge.node {
                SidecarNode::Image { id, display_url, .. } => Some(Ok(InstagramMediaItem {
                    id,
                    media_type: MediaType::Image,
                    url: display_url.clone(),
                    thumbnail_url: display_url.clone(),
                    timestamp: DateTime::from_timestamp(sidecar.taken_at_timestamp, 0).unwrap(),
                })),
                SidecarNode::Video {
                    id,
                    video_url,
                    display_url,
                    ..
                } => Some(Ok(InstagramMediaItem {
                    id,
                    media_type: MediaType::Video,
                    url: video_url.clone(),
                    thumbnail_url: display_url.clone(),
                    timestamp: DateTime::from_timestamp(sidecar.taken_at_timestamp, 0).unwrap(),
                })),
            })
            .collect::<Result<Vec<_>, InstagramError>>()?;

        Ok(InstagramMedia {
            id: sidecar.id,
            shortcode: sidecar.shortcode,
            author: InstagramAuthor {
                id: sidecar.owner.id,
                username: sidecar.owner.username,
            },
            thumbnail_url: items.first().unwrap().thumbnail_url.clone(),
            timestamp: DateTime::from_timestamp(sidecar.taken_at_timestamp, 0)
                .context("Failed to parse timestamp")
                .unwrap(),
            caption: sidecar
                .edge_media_to_caption
                .edges
                .first()
                .map(|edge| edge.node.text.clone()),
            // TODO: only take first 50 characters, use Cow as much as possible to reduce clone
            content: InstagramContent::Multiple(items),
        })
    }
}

impl TryFrom<GraphStoryItem> for InstagramMedia {
    type Error = InstagramError;

    fn try_from(story: GraphStoryItem) -> Result<Self, Self::Error> {
        let (id, media_type, url, timestamp, owner, thumbnail_url) = match story {
            GraphStoryItem::Image(GraphStoryItemImage {
                id,
                display_url,
                taken_at_timestamp,
                owner,
                ..
            }) => (
                id,
                MediaType::Image,
                display_url.clone(),
                taken_at_timestamp,
                owner,
                display_url,
            ),
            GraphStoryItem::Video(GraphStoryItemVideo {
                id,
                video_resources,
                taken_at_timestamp,
                owner,
                display_url,
                ..
            }) => {
                let video_url = video_resources
                    .first()
                    .ok_or_else(|| InstagramError::InvalidUrl("No video resources found".into()))?
                    .src
                    .clone();
                (id, MediaType::Video, video_url, taken_at_timestamp, owner, display_url)
            }
        };

        Ok(InstagramMedia {
            id: id.clone(),
            shortcode: id.clone(),
            author: InstagramAuthor {
                id: owner.id,
                username: owner.username,
            },
            caption: None,
            thumbnail_url: thumbnail_url.clone(),
            timestamp: DateTime::from_timestamp(timestamp, 0)
                .context("Failed to parse timestamp")
                .unwrap(),
            content: InstagramContent::Story(InstagramMediaItem {
                id,
                media_type,
                url,
                thumbnail_url,
                timestamp: DateTime::from_timestamp(timestamp, 0)
                    .context("Failed to parse timestamp")
                    .unwrap(),
            }),
        })
    }
}

// --- Common ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    pub id: String,
    pub username: String,
}

// --- XDTGraphMedia ---
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum XDTGraphMedia {
    #[serde(rename = "XDTGraphImage")]
    Image(XDTGraphImage),
    #[serde(rename = "XDTGraphVideo")]
    Video(XDTGraphVideo),
    #[serde(rename = "XDTGraphSidecar")]
    Sidecar(XDTGraphSidecar),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XDTGraphImage {
    pub id: String,
    pub shortcode: String,
    pub display_url: String,
    pub owner: Owner,
    pub edge_media_to_caption: EdgeMediaToCaption,
    pub taken_at_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XDTGraphVideo {
    pub id: String,
    pub shortcode: String,
    pub display_url: String,
    pub video_url: String,
    pub owner: Owner,
    pub edge_media_to_caption: EdgeMediaToCaption,
    pub taken_at_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XDTGraphSidecar {
    pub id: String,
    pub shortcode: String,
    pub owner: Owner,
    pub edge_media_to_caption: EdgeMediaToCaption,
    pub taken_at_timestamp: i64,
    pub edge_sidecar_to_children: EdgeSidecarToChildren,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMediaToCaption {
    pub edges: Vec<EdgeMediaToCaptionEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMediaToCaptionEdge {
    pub node: EdgeMediaToCaptionNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMediaToCaptionNode {
    pub id: String,
    pub created_at: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeSidecarToChildren {
    pub edges: Vec<SidecarEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarEdge {
    pub node: SidecarNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum SidecarNode {
    #[serde(rename = "XDTGraphImage")]
    Image {
        id: String,
        shortcode: String,
        display_url: String,
        #[serde(default)]
        is_video: bool,
    },
    #[serde(rename = "XDTGraphVideo")]
    Video {
        id: String,
        shortcode: String,
        display_url: String,
        video_url: String,
        #[serde(default)]
        is_video: bool,
    },
}

// ---- Story ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphReel {
    #[serde(rename = "__typename")]
    pub typename: String,
    pub id: String,
    pub items: Vec<GraphStoryItem>,
    pub owner: Owner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum GraphStoryItem {
    #[serde(rename = "GraphStoryImage")]
    Image(GraphStoryItemImage),
    #[serde(rename = "GraphStoryVideo")]
    Video(GraphStoryItemVideo),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStoryItemImage {
    pub id: String,
    pub display_url: String,
    pub taken_at_timestamp: i64,
    pub owner: Owner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStoryItemVideo {
    pub id: String,
    pub display_url: String,
    pub video_resources: Vec<VideoResource>,
    pub taken_at_timestamp: i64,
    pub owner: Owner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoResource {
    pub src: String,
    pub profile: Option<String>,
    pub config_width: Option<i32>,
    pub config_height: Option<i32>,
}
