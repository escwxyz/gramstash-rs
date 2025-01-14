use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

use super::auth::SessionData;
use crate::{
    error::{BotError, BotResult, InstagramError, ServiceError},
    state::AppState,
    utils::http,
};

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
    // pub timestamp: Option<String>,
    // pub expires_at: Option<String>,
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
    Reel { shortcode: String },
}

#[derive(Clone)]
pub struct InstagramService {
    pub public_client: reqwest::Client,
}

impl InstagramService {
    pub fn new() -> BotResult<Self> {
        let builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(30))
            .default_headers(http::build_instagram_headers())
            .user_agent(http::INSTAGRAM_USER_AGENT);

        let public_client = http::build_client(builder)?;
        Ok(Self { public_client })
    }
    #[allow(dead_code)]
    pub fn with_session(_session_data: SessionData) -> BotResult<Self> {
        // TODO: Create a new client with session cookies
        todo!()
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
                shortcode: story_id.to_string(),
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

    pub async fn fetch_post_info(&self, shortcode: &str) -> BotResult<MediaInfo> {
        let state = AppState::get()?;

        let api_url = state.config.instagram.api_endpoint.clone();
        let doc_id = state.config.instagram.doc_id.clone();

        let body = serde_json::json!({
            "doc_id": doc_id,
            "variables": {
                "shortcode": shortcode
            }
        });

        let response = self
            .public_client
            .post(&api_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| BotError::ServiceError(ServiceError::InstagramError(InstagramError::NetworkError(e))))?;

        if !response.status().is_success() {
            return Err(BotError::ServiceError(ServiceError::InstagramError(
                InstagramError::ApiError(format!("Instagram API returned status: {}", response.status())),
            )));
        }

        let data: serde_json::Value = response.json().await.map_err(|e| {
            BotError::ServiceError(ServiceError::InstagramError(InstagramError::DeserializationError(
                e.to_string(),
            )))
        })?;

        self.parse_media_response(data)
    }

    fn parse_media_response(&self, data: serde_json::Value) -> BotResult<MediaInfo> {
        let media = data
            .get("data")
            .and_then(|d| d.get("xdt_shortcode_media"))
            .ok_or_else(|| {
                BotError::ServiceError(ServiceError::InstagramError(InstagramError::InvalidResponseStructure(
                    "Missing xdt_shortcode_media".to_string(),
                )))
            })?;

        let typename = media.get("__typename").and_then(|t| t.as_str()).ok_or_else(|| {
            BotError::ServiceError(ServiceError::InstagramError(InstagramError::InvalidResponseStructure(
                "Missing typename".to_string(),
            )))
        })?;

        match typename {
            "XDTGraphImage" => self.parse_image(media),
            "XDTGraphVideo" => self.parse_reel(media),
            "XDTGraphSidecar" => self.parse_carousel(media),
            _ => Err(BotError::ServiceError(ServiceError::InstagramError(
                InstagramError::InvalidResponseStructure(format!("Unspported media type: {}", typename)),
            ))),
        }
    }

    fn parse_reel(&self, media: &serde_json::Value) -> BotResult<MediaInfo> {
        let url = media
            .get("video_url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| BotError::InvalidUrl("Missing video URL".into()))?
            .to_string();

        Ok(MediaInfo {
            content: MediaContent::Post(PostContent::Single {
                url: Url::parse(&url).map_err(|e| BotError::InvalidUrl(e.to_string()))?,
                content_type: ContentType::Reel,
                // caption: None,
                // timestamp: None,
            }),
            author: self.get_author(media)?,
        })
    }

    fn parse_image(&self, media: &serde_json::Value) -> BotResult<MediaInfo> {
        let url = self.find_display_url(media)?;

        Ok(MediaInfo {
            content: MediaContent::Post(PostContent::Single {
                url: Url::parse(&url).map_err(|e| BotError::InvalidUrl(e.to_string()))?,
                content_type: ContentType::Image,
                // caption: None,
            }),
            author: self.get_author(media)?,
        })
    }

    fn parse_carousel_video(&self, node: &serde_json::Value) -> BotResult<CarouselItem> {
        let url = node
            .get("video_url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| {
                BotError::ServiceError(ServiceError::InstagramError(InstagramError::InvalidResponseStructure(
                    "Missing carousel video URL".to_string(),
                )))
            })?
            .to_string();
        Ok(CarouselItem {
            url: Url::parse(&url).map_err(|e| BotError::InvalidUrl(e.to_string()))?,
            content_type: ContentType::Reel,
            // caption: None,
        })
    }

    fn parse_carousel_image(&self, node: &serde_json::Value) -> BotResult<CarouselItem> {
        let url = self.find_display_url(node)?;
        Ok(CarouselItem {
            url: Url::parse(&url).map_err(|e| BotError::InvalidUrl(e.to_string()))?,
            content_type: ContentType::Image,
            // caption: None,
            // timestamp: None,
        })
    }

    fn parse_carousel(&self, media: &serde_json::Value) -> BotResult<MediaInfo> {
        info!("Parsing carousel ...");
        let edges = media
            .get("edge_sidecar_to_children")
            .and_then(|e| e.get("edges"))
            .and_then(|e| e.as_array())
            .ok_or_else(|| BotError::InvalidUrl("Missing carousel edges".into()))?;

        let mut carousel_items = Vec::new();
        for edge in edges {
            let node = edge
                .get("node")
                .ok_or_else(|| BotError::InvalidUrl("Missing node".into()))?;
            let is_video = node.get("is_video").and_then(|v| v.as_bool()).unwrap_or(false);

            let item = if is_video {
                self.parse_carousel_video(node)?
            } else {
                self.parse_carousel_image(node)?
            };

            carousel_items.push(item);
        }

        Ok(MediaInfo {
            content: MediaContent::Post(PostContent::Carousel { items: carousel_items }),
            author: self.get_author(media)?,
        })
    }

    fn get_author(&self, media: &serde_json::Value) -> BotResult<MediaAuthor> {
        let username = media
            .get("owner")
            .and_then(|o| o.get("username"))
            .and_then(|u| u.as_str())
            .unwrap_or("unknown");

        Ok(MediaAuthor {
            username: username.to_string(),
        })
    }

    fn get_dimensions(&self, media: &serde_json::Value) -> BotResult<(u64, u64)> {
        let width = media
            .get("dimensions")
            .and_then(|d| d.get("width"))
            .and_then(|w| w.as_u64())
            .unwrap_or(1080);

        let height = media
            .get("dimensions")
            .and_then(|d| d.get("height"))
            .and_then(|h| h.as_u64())
            .unwrap_or(1080);

        Ok((width, height))
    }

    fn find_display_url(&self, media_or_node: &serde_json::Value) -> BotResult<String> {
        let (width, height) = self.get_dimensions(media_or_node)?;

        let url = media_or_node
            .get("display_resources")
            .and_then(|resources| resources.as_array())
            .and_then(|resources| {
                resources
                    .iter()
                    .find(|resource| {
                        let res_width = resource.get("config_width").and_then(|w| w.as_u64()).unwrap_or(0);
                        let res_height = resource.get("config_height").and_then(|h| h.as_u64()).unwrap_or(0);
                        res_width == width && res_height == height
                    })
                    .or_else(|| resources.last())
                    .and_then(|resource| resource.get("src"))
                    .and_then(|u| u.as_str())
            })
            .unwrap_or_else(|| media_or_node.get("display_url").and_then(|u| u.as_str()).unwrap_or(""))
            .to_string();

        Ok(url)
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
    fn test_get_dimensions() {
        let service = create_service();
        let sample_response = load_test_data("image_post.json");
        let media = sample_response
            .get("data")
            .and_then(|d| d.get("xdt_shortcode_media"))
            .unwrap();
        let result = service.get_dimensions(&media);
        assert!(result.is_ok());
        let (width, height) = result.unwrap();
        assert_eq!(width, 750);
        assert_eq!(height, 938);
    }

    #[test]
    fn test_get_display_url() {
        let service = create_service();
        let sample_response = load_test_data("image_post.json");
        let media = sample_response
            .get("data")
            .and_then(|d| d.get("xdt_shortcode_media"))
            .unwrap();
        let result = service.find_display_url(&media);
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url, "https://scontent.cdninstagram.com/v/t51.29350-15/472967659_1558220318165692_1609720637204509458_n.jpg?stp=dst-jpg_e35_tt6&efg=eyJ2ZW5jb2RlX3RhZyI6ImltYWdlX3VybGdlbi42NjZ4ODMzLnNkci5mMjkzNTAuZGVmYXVsdF9pbWFnZSJ9&_nc_ht=scontent.cdninstagram.com&_nc_cat=104&_nc_ohc=hWqiRXngiZEQ7kNvgHR3cM1&_nc_gid=a733a617b0d34e25813ee7272d21b098&edm=ANTKIIoBAAAA&ccb=7-5&oh=00_AYDAtqG1LkHKtCNRn52TiWRDvmpF_V3AcqeVgKWGOa1d-A&oe=6788F599&_nc_sid=d885a2".to_string());
    }

    #[test]
    fn test_get_author() {
        let service = create_service();
        let sample_response = load_test_data("image_post.json");
        let media = sample_response
            .get("data")
            .and_then(|d| d.get("xdt_shortcode_media"))
            .unwrap();
        let result = service.get_author(&media);
        assert!(result.is_ok());
        let author = result.unwrap();
        assert_eq!(author.username, "unownedspaces".to_string());
    }

    #[test]
    fn test_parse_image_post() {
        let service = create_service();
        let sample_response = load_test_data("image_post.json");

        let result = service.parse_media_response(sample_response);
        assert!(result.is_ok());
        let media_info = result.unwrap();
        match media_info.content {
            MediaContent::Post(PostContent::Single { url, content_type, .. }) => {
                assert_eq!(content_type, ContentType::Image);

                assert_eq!(url.to_string(), "https://scontent.cdninstagram.com/v/t51.29350-15/472967659_1558220318165692_1609720637204509458_n.jpg?stp=dst-jpg_e35_tt6&efg=eyJ2ZW5jb2RlX3RhZyI6ImltYWdlX3VybGdlbi42NjZ4ODMzLnNkci5mMjkzNTAuZGVmYXVsdF9pbWFnZSJ9&_nc_ht=scontent.cdninstagram.com&_nc_cat=104&_nc_ohc=hWqiRXngiZEQ7kNvgHR3cM1&_nc_gid=a733a617b0d34e25813ee7272d21b098&edm=ANTKIIoBAAAA&ccb=7-5&oh=00_AYDAtqG1LkHKtCNRn52TiWRDvmpF_V3AcqeVgKWGOa1d-A&oe=6788F599&_nc_sid=d885a2".to_string())
            }
            _ => panic!("Expected Single Image post"),
        }
    }

    #[test]
    fn test_parse_reel_post() {
        let service = create_service();
        let sample_response = load_test_data("reel_post.json");

        let result = service.parse_media_response(sample_response);
        assert!(result.is_ok());
        let media_info = result.unwrap();
        match media_info.content {
            MediaContent::Post(PostContent::Single { url, content_type, .. }) => {
                assert_eq!(content_type, ContentType::Reel);
                assert_eq!(url.to_string(), "https://scontent.cdninstagram.com/o1/v/t16/f2/m86/AQMfVTMYUej1SuiM5cnf_mB5sRbj3y0OHcma_t_QSYhVB9o6KlnkTfPv2YYT2KkzNv6S-4wlrNyRvBULyivzkcY7wUFH3eRZiskh3CQ.mp4?stp=dst-mp4&efg=eyJxZV9ncm91cHMiOiJbXCJpZ193ZWJfZGVsaXZlcnlfdnRzX290ZlwiXSIsInZlbmNvZGVfdGFnIjoidnRzX3ZvZF91cmxnZW4uY2xpcHMuYzIuNzIwLmJhc2VsaW5lIn0&_nc_cat=109&vs=1303164467384864_2268861127&_nc_vs=HBksFQIYUmlnX3hwdl9yZWVsc19wZXJtYW5lbnRfc3JfcHJvZC9FRjRENzJBRUExNzM1MjA0RTZGQTVEODNEQTIyRjg5Nl92aWRlb19kYXNoaW5pdC5tcDQVAALIAQAVAhg6cGFzc3Rocm91Z2hfZXZlcnN0b3JlL0dGVFVIQnhfTGFxXy1Gb0RBRWMzazNJOUxjVjFicV9FQUFBRhUCAsgBACgAGAAbABUAACbKzNW4g8SYQBUCKAJDMywXQBjMzMzMzM0YEmRhc2hfYmFzZWxpbmVfMV92MREAdf4HAA%3D%3D&ccb=9-4&oh=00_AYA9KqvLmWCLvQGQFZzPjBUN6HXVe3A5zrIfOG_TjWuqzA&oe=677BD5AB&_nc_sid=d885a2".to_string());
            }
            _ => panic!("Expected Single Reel post"),
        }
    }

    #[test]
    fn test_parse_carousel_post() {
        let service = create_service();
        let sample_response = load_test_data("carousel_post.json");

        let result = service.parse_media_response(sample_response);
        assert!(result.is_ok());
        let media_info = result.unwrap();
        match media_info.content {
            MediaContent::Post(PostContent::Carousel { items }) => {
                assert!(!items.is_empty(), "Carousel should contain items");
                assert!(matches!(items[0].content_type, ContentType::Image | ContentType::Reel));
                assert_eq!(items[0].url.to_string(), "https://scontent.cdninstagram.com/v/t51.29350-15/472483388_1743296202904144_2127040402149707726_n.jpg?stp=dst-jpegr_e35_s1080x1080_tt6&efg=eyJ2ZW5jb2RlX3RhZyI6ImltYWdlX3VybGdlbi4xMjAweDkwMC5oZHIuZjI5MzUwLmRlZmF1bHRfaW1hZ2UifQ&_nc_ht=scontent.cdninstagram.com&_nc_cat=104&_nc_ohc=H7FAsYqUt8sQ7kNvgHKQueD&_nc_gid=454829c2394e4760b0076d2770ff8bcd&edm=ANTKIIoBAAAA&ccb=7-5&oh=00_AYANAapQhxeFbA3f6N2AcgZEtnKHKjJhvkweQ4kX7PZr6A&oe=6788DF75&_nc_sid=d885a2".to_string());
            }
            _ => panic!("Expected Carousel post"),
        }
    }
}
