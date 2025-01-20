use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::{
    config::AppConfig,
    error::{BotError, BotResult, InstagramError, ServiceError},
    services::http::{DeviceType, HttpService},
};

use super::cache::CacheService;

#[derive(Debug, Clone, PartialEq)]
pub enum InstagramIdentifier {
    Story { username: String, story_id: String },
    Post { shortcode: String },
    Reel { shortcode: String },
}

// --- ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstagramMedia {
    // Common metadata
    pub id: String,
    pub shortcode: Option<String>, // Stories don't have shortcode
    pub author: InstagramAuthor,
    pub timestamp: DateTime<Utc>,
    pub caption: Option<String>,
    pub content: InstagramContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstagramAuthor {
    pub id: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstagramContent {
    Single(MediaItem),
    Multiple(Vec<MediaItem>),
    Story(MediaItem),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: String,
    pub media_type: MediaType,
    pub url: Url,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MediaType {
    Image,
    Video,
}

impl TryFrom<XDTGraphImage> for InstagramMedia {
    type Error = BotError;

    fn try_from(image: XDTGraphImage) -> Result<Self, Self::Error> {
        Ok(InstagramMedia {
            id: image.id.clone(),
            shortcode: Some(image.shortcode),
            author: InstagramAuthor {
                id: image.owner.id,
                username: image.owner.username,
            },
            timestamp: DateTime::from_timestamp(image.taken_at_timestamp, 0).unwrap(),
            caption: image
                .edge_media_to_caption
                .edges
                .first()
                .map(|edge| edge.node.text.clone()),
            content: InstagramContent::Single(MediaItem {
                id: image.id.clone(),
                media_type: MediaType::Image,
                url: Url::parse(&image.display_url).map_err(|e| BotError::InvalidUrl(e.to_string()))?,
            }),
        })
    }
}

impl TryFrom<XDTGraphVideo> for InstagramMedia {
    type Error = BotError;

    fn try_from(video: XDTGraphVideo) -> Result<Self, Self::Error> {
        Ok(InstagramMedia {
            id: video.id.clone(),
            shortcode: Some(video.shortcode),
            author: InstagramAuthor {
                id: video.owner.id,
                username: video.owner.username,
            },
            timestamp: DateTime::from_timestamp(video.taken_at_timestamp, 0).unwrap(),
            caption: video
                .edge_media_to_caption
                .edges
                .first()
                .map(|edge| edge.node.text.clone()),
            content: InstagramContent::Single(MediaItem {
                id: video.id.clone(),
                media_type: MediaType::Video,
                url: Url::parse(&video.video_url).map_err(|e| BotError::InvalidUrl(e.to_string()))?,
            }),
        })
    }
}

impl TryFrom<XDTGraphSidecar> for InstagramMedia {
    type Error = BotError;

    fn try_from(sidecar: XDTGraphSidecar) -> Result<Self, Self::Error> {
        let items = sidecar
            .edge_sidecar_to_children
            .edges
            .into_iter()
            .filter_map(|edge| match edge.node {
                SidecarNode::Image { id, display_url, .. } => Some(Ok(MediaItem {
                    id,
                    media_type: MediaType::Image,
                    url: Url::parse(&display_url).ok()?,
                })),
                SidecarNode::Video { id, video_url, .. } => Some(Ok(MediaItem {
                    id,
                    media_type: MediaType::Video,
                    url: Url::parse(&video_url).ok()?,
                })),
            })
            .collect::<Result<Vec<_>, BotError>>()?;

        Ok(InstagramMedia {
            id: sidecar.id,
            shortcode: Some(sidecar.shortcode),
            author: InstagramAuthor {
                id: sidecar.owner.id,
                username: sidecar.owner.username,
            },
            timestamp: DateTime::from_timestamp(sidecar.taken_at_timestamp, 0).unwrap(),
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
    type Error = BotError;

    fn try_from(story: GraphStoryItem) -> Result<Self, Self::Error> {
        let (id, media_type, url, timestamp) = match story {
            GraphStoryItem::Image(GraphStoryItemImage {
                id,
                display_url,
                taken_at_timestamp,
                ..
            }) => (id, MediaType::Image, display_url, taken_at_timestamp),
            GraphStoryItem::Video(GraphStoryItemVideo {
                id,
                video_resources,
                taken_at_timestamp,
                ..
            }) => {
                let video_url = video_resources
                    .first()
                    .ok_or_else(|| BotError::InvalidUrl("No video resources found".into()))?
                    .src
                    .clone();
                (id, MediaType::Video, video_url, taken_at_timestamp)
            }
        };

        Ok(InstagramMedia {
            id: id.clone(),
            shortcode: None,
            author: InstagramAuthor {
                id: String::new(), // Story items don't contain author info directly
                username: String::new(),
            },
            timestamp: DateTime::from_timestamp(timestamp, 0).unwrap(),
            caption: None,
            content: InstagramContent::Story(MediaItem {
                id,
                media_type,
                url: Url::parse(&url).map_err(|e| BotError::InvalidUrl(e.to_string()))?,
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
    // pub expiring_at_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStoryItemVideo {
    pub id: String,
    pub display_url: String,
    pub video_resources: Vec<VideoResource>,
    pub taken_at_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoResource {
    pub src: String,
    pub profile: Option<String>,
    pub config_width: Option<i32>,
    pub config_height: Option<i32>,
}

// --- END ---

#[derive(Clone)]
pub struct InstagramService {
    http: HttpService,
}

impl InstagramService {
    pub fn new() -> BotResult<Self> {
        info!("Initializing InstagramService...");
        let http = HttpService::new(true, DeviceType::Desktop, None)?;
        info!("InstagramService initialized");
        Ok(Self { http })
    }

    // async fn graphql_query(&self, query_hash: &str, variables: Value) -> BotResult<Value> {
    //     let params = serde_json::json!({
    //         "query_hash": query_hash,
    //         "variables": variables.to_string(),
    //     });

    //     self.http.get_json("graphql/query", &params, None, None, false).await
    // }

    async fn doc_id_graphql_query(&self, doc_id: &str, variables: Value) -> BotResult<Value> {
        let params = serde_json::json!({
            "doc_id": doc_id,
            "variables": variables.to_string(),
            "server_timestamps": "true",
        });

        self.http.get_json("graphql/query", &params, None, None, true).await
    }

    pub fn parse_instagram_url(&self, url: &Url) -> BotResult<InstagramIdentifier> {
        let path_segments: Vec<_> = url
            .path_segments()
            .ok_or_else(|| BotError::InvalidUrl("No path segments found".into()))?
            .collect();

        info!("Parsing Instagram URL with path segments: {:?}", path_segments);

        match path_segments.as_slice() {
            ["stories", username, story_id] => Ok(InstagramIdentifier::Story {
                username: username.to_string(),
                story_id: story_id.to_string(),
            }),
            ["p", shortcode, ..] => Ok(InstagramIdentifier::Post {
                shortcode: shortcode.to_string(),
            }),
            ["reel", shortcode, ..] => Ok(InstagramIdentifier::Reel {
                shortcode: shortcode.to_string(),
            }),
            _ => Err(BotError::InvalidUrl("Invalid Instagram URL format".into())),
        }
    }

    pub async fn fetch_post_info(&self, shortcode: &str) -> BotResult<InstagramMedia> {
        info!("Fetching post info for shortcode: {}", shortcode);

        let config = AppConfig::get()?;
        let doc_id = config.instagram.doc_id.clone();

        let variables = serde_json::json!({
            "shortcode": shortcode
        });

        let data = self.doc_id_graphql_query(&doc_id, variables).await?;
        self.parse_post_response(data)
    }

    pub async fn get_story(
        &self,
        telegram_user_id: &str,
        target_instagram_username: &str,
        story_id: &str,
        http: &HttpService,
    ) -> BotResult<InstagramMedia> {
        if let Some(cached) = CacheService::get_media_from_redis(telegram_user_id, story_id).await? {
            return Ok(cached);
        }

        // Fetch all user ids or only the target user id
        let user_ids = self.fetch_user_ids(http, Some(target_instagram_username)).await?;

        // Fetch all stories belonging to the user ids
        let stories = self.fetch_stories(http, user_ids).await?;

        if let Some(reel) = stories.first() {
            for item in reel.items.iter() {
                let media: InstagramMedia = item.clone().try_into()?;
                CacheService::cache_media_to_redis(telegram_user_id, target_instagram_username, &media.id, &media)
                    .await?;

                if media.id == story_id {
                    return Ok(media);
                }
            }
        }

        // If we get here, the story wasn't found
        Err(BotError::ServiceError(ServiceError::InstagramError(
            InstagramError::InvalidResponseStructure(format!(
                "Story not found for user {} with id {}",
                target_instagram_username, story_id
            )),
        )))
    }

    /// Fetch user IDs from the stories feed
    async fn fetch_user_ids(
        &self,
        http: &HttpService,
        target_instagram_username: Option<&str>,
    ) -> BotResult<Vec<String>> {
        info!("Fetching user IDs...");
        let variables = serde_json::json!({
            "only_stories": true
        });

        let params = serde_json::json!({
            "query_hash": "d15efd8c0c5b23f0ef71f18bf363c704",
            "variables": variables.to_string(),
        });

        let stories_data = http.get_json("graphql/query", &params, None, None, true).await?;

        let edges = stories_data["data"]["user"]["feed_reels_tray"]["edge_reels_tray_to_reel"]["edges"]
            .as_array()
            .ok_or_else(|| {
                BotError::ServiceError(ServiceError::InstagramError(InstagramError::InvalidResponseStructure(
                    "Failed to get edges from response".to_string(),
                )))
            })?;

        // {
        //     "node": {
        //       "can_reply": true,
        //       "expiring_at": 1737431150,
        //       "id": "2267629874",
        //       "latest_reel_media": 1737295160,
        //       "muted": false,
        //       "prefetch_count": 0,
        //       "ranked_position": 2,
        //       "seen": null,
        //       "seen_ranked_position": 2,
        //       "user": {
        //         "id": "2267629874",
        //         "profile_pic_url": "https://scontent-lax3-2.cdninstagram.com/v/t51.2885-19/462178502_1075515874120935_2177629506287086586_n.jpg?stp=dst-jpg_s150x150_tt6&_nc_ht=scontent-lax3-2.cdninstagram.com&_nc_cat=103&_nc_ohc=jT9Z65UUNlQQ7kNvgEFuG-i&_nc_gid=3bf0da0658334e07baa3647812cf6c9e&edm=APrQDZQBAAAA&ccb=7-5&oh=00_AYBJQoqI8fWIsVpysHZ8v6VSQfl66ebI8QVu1zG1DuZIQQ&oe=67937F46&_nc_sid=01a934",
        //         "username": "yvweii_"
        //       }
        //     }
        //   },

        // Use iterator chaining for cleaner collection
        let user_ids: Vec<String> = edges
            .iter()
            .filter_map(|edge| {
                let id = edge["node"]["id"].as_str()?;
                let username = edge["node"]["user"]["username"].as_str()?;

                match target_instagram_username {
                    Some(target) if username == target => Some(id.to_string()),
                    None => Some(id.to_string()),
                    _ => None,
                }
            })
            .collect();

        Ok(user_ids)
    }

    /// Fetch stories based on user IDs
    pub async fn fetch_stories(
        &self,
        http: &HttpService,
        instagram_user_ids: Vec<String>,
    ) -> BotResult<Vec<GraphReel>> {
        let variables = serde_json::json!({
            "reel_ids": instagram_user_ids,
            "precomposed_overlay": false
        });

        let params = serde_json::json!({
            "query_hash": "303a4ae99711322310f25250d988f3b7",
            "variables": variables.to_string(),
        });

        let data = http.get_json("graphql/query", &params, None, None, true).await?;

        let stories = self.parse_stories_response(data)?;

        Ok(stories)
    }

    fn parse_stories_response(&self, data: Value) -> BotResult<Vec<GraphReel>> {
        let reels_media = data
            .get("data")
            .and_then(|d| d.get("reels_media"))
            .and_then(|r| r.as_array())
            .ok_or_else(|| {
                BotError::ServiceError(ServiceError::InstagramError(InstagramError::InvalidResponseStructure(
                    "Missing reels_media data".to_string(),
                )))
            })?;

        let reels: Vec<GraphReel> =
            serde_json::from_value::<Vec<GraphReel>>(serde_json::Value::Array(reels_media.to_vec())).map_err(|e| {
                BotError::ServiceError(ServiceError::InstagramError(InstagramError::DeserializationError(
                    format!("Failed to deserialize reels: {}", e),
                )))
            })?;

        Ok(reels)
    }

    fn parse_post_response(&self, data: Value) -> BotResult<InstagramMedia> {
        info!("Parsing post response ...");
        let media = data
            .get("data")
            .and_then(|d| d.get("xdt_shortcode_media"))
            .ok_or_else(|| {
                BotError::ServiceError(ServiceError::InstagramError(InstagramError::InvalidResponseStructure(
                    "Missing xdt_shortcode_media".to_string(),
                )))
            })?;

        let media_data = serde_json::from_value::<XDTGraphMedia>(media.clone()).map_err(|e| {
            BotError::ServiceError(ServiceError::InstagramError(InstagramError::DeserializationError(
                format!("Failed to deserialize media: {}", e),
            )))
        })?;

        match media_data {
            XDTGraphMedia::Image(media) => self.parse_image(media),
            XDTGraphMedia::Video(media) => self.parse_reel(media),
            XDTGraphMedia::Sidecar(sidecar) => self.parse_sidecar(sidecar),
        }
    }

    fn parse_reel(&self, media: XDTGraphVideo) -> BotResult<InstagramMedia> {
        Ok(media.try_into()?)
    }

    fn parse_image(&self, media: XDTGraphImage) -> BotResult<InstagramMedia> {
        Ok(media.try_into()?)
    }

    fn parse_sidecar(&self, media: XDTGraphSidecar) -> BotResult<InstagramMedia> {
        Ok(media.try_into()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn create_service() -> InstagramService {
        InstagramService::new().unwrap()
    }

    fn load_test_data(filename: &str) -> serde_json::Value {
        let path = Path::new("tests/data/instagram").join(filename);
        let data = fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read test data file: {}", filename));
        serde_json::from_str(&data).unwrap_or_else(|_| panic!("Failed to parse JSON from file: {}", filename))
    }

    #[test]
    fn test_parse_image() {
        let service = create_service();
        let sample_response = load_test_data("image_post.json");

        let result = service.parse_post_response(sample_response);
        assert!(result.is_ok());
        let instagram_media = result.unwrap();
        assert_eq!(instagram_media.shortcode, Some("DEpxownyoJf".to_string()));
        assert_eq!(instagram_media.author.username, "unownedspaces".to_string());
        assert!(matches!(instagram_media.content, InstagramContent::Single(_)));
    }

    #[test]
    fn test_parse_reel() {
        let service = create_service();
        let sample_response = load_test_data("reel_post.json");

        let result = service.parse_post_response(sample_response);
        assert!(result.is_ok());
        let instagram_media = result.unwrap();
        assert_eq!(instagram_media.shortcode, Some("DEQCBZcPVEY".to_string()));

        match instagram_media.content {
            InstagramContent::Single(MediaItem { id, media_type, url }) => {
                assert_eq!(url.to_string(), "https://scontent.cdninstagram.com/o1/v/t16/f2/m86/AQMfVTMYUej1SuiM5cnf_mB5sRbj3y0OHcma_t_QSYhVB9o6KlnkTfPv2YYT2KkzNv6S-4wlrNyRvBULyivzkcY7wUFH3eRZiskh3CQ.mp4?stp=dst-mp4&efg=eyJxZV9ncm91cHMiOiJbXCJpZ193ZWJfZGVsaXZlcnlfdnRzX290ZlwiXSIsInZlbmNvZGVfdGFnIjoidnRzX3ZvZF91cmxnZW4uY2xpcHMuYzIuNzIwLmJhc2VsaW5lIn0&_nc_cat=109&vs=1303164467384864_2268861127&_nc_vs=HBksFQIYUmlnX3hwdl9yZWVsc19wZXJtYW5lbnRfc3JfcHJvZC9FRjRENzJBRUExNzM1MjA0RTZGQTVEODNEQTIyRjg5Nl92aWRlb19kYXNoaW5pdC5tcDQVAALIAQAVAhg6cGFzc3Rocm91Z2hfZXZlcnN0b3JlL0dGVFVIQnhfTGFxXy1Gb0RBRWMzazNJOUxjVjFicV9FQUFBRhUCAsgBACgAGAAbABUAACbKzNW4g8SYQBUCKAJDMywXQBjMzMzMzM0YEmRhc2hfYmFzZWxpbmVfMV92MREAdf4HAA%3D%3D&ccb=9-4&oh=00_AYA9KqvLmWCLvQGQFZzPjBUN6HXVe3A5zrIfOG_TjWuqzA&oe=677BD5AB&_nc_sid=d885a2".to_string());
                assert_eq!(
                    url.to_string(),
                    "https://scontent.cdninstagram.com/v/t51.2885-15/472307658_18489647257009598_4338556116685713421_n.jpg?stp=dst-jpg_e15_tt6&_nc_ht=scontent.cdninstagram.com&_nc_cat=102&_nc_ohc=jDa7898fzrwQ7kNvgGNR9hq&_nc_gid=76dc0ba7a05e451d9bddd368e2c93b28&edm=ANTKIIoBAAAA&ccb=7-5&oh=00_AYALRcW2l00E25rRwLBAc2LIznpxa0fQlaA1DRXDFRnK6Q&oe=677FC141&_nc_sid=d885a2".to_string()
                );
                assert_eq!(media_type, MediaType::Video);
                assert_eq!(id, "3535334599615664408");
            }
            _ => panic!("Expected single reel post"),
        }
    }

    #[test]
    fn test_parse_sidecar() {
        let service = create_service();
        let sample_response = load_test_data("carousel_post.json");

        let result = service.parse_post_response(sample_response);
        assert!(result.is_ok());
        let instagram_media = result.unwrap();
        assert_eq!(instagram_media.shortcode, Some("DEr8_LbMn2P".to_string()));
        assert_eq!(instagram_media.author.username, "hannah__dasilva".to_string());
        assert!(matches!(instagram_media.content, InstagramContent::Multiple(_)));
        match instagram_media.content {
            InstagramContent::Multiple(media_items) => {
                assert_eq!(media_items.len(), 5);
            }
            _ => panic!("Expected multiple carousel post"),
        }
    }
    #[test]
    fn test_parse_stories_response() {
        let service = create_service();
        let sample_response = load_test_data("stories.json");

        let result = service.parse_stories_response(sample_response);
        assert!(result.is_ok());
        let reels = result.unwrap();
        assert!(!reels.is_empty());
        assert_eq!(reels[0].id, "770605631");
        assert_eq!(reels[0].owner.username, "st.einberg");
    }
}
