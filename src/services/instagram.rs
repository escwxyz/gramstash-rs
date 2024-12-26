use crate::utils::error::BotError;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaType {
    Image,
    Video,
    Carousel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub url: String,
    pub media_type: MediaType,
    pub file_size: u64,
    pub carousel_items: Vec<CarouselItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarouselItem {
    pub url: String,
    pub media_type: MediaType,
    pub file_size: u64,
}

pub struct InstagramService {
    client: Client,
}

impl InstagramService {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub async fn get_media_info(&self, url: &str) -> Result<MediaInfo, BotError> {
        // Parse and validate the URL
        // TODO:
        let parsed_url = Url::parse(url).map_err(|_| BotError::InvalidUrl("Invalid Instagram URL".into()))?;

        // Extract post ID from URL
        let post_id = self.extract_post_id(&parsed_url)?;

        // Get the media info using Instagram's API
        let media_info = self.fetch_media_info(&post_id).await?;

        Ok(media_info)
    }

    async fn fetch_media_info(&self, post_id: &str) -> Result<MediaInfo, BotError> {
        // Instagram GraphQL API endpoint
        // TODO: need to test this
        let api_url = format!(
            "https://www.instagram.com/graphql/query/?query_hash={}&variables={}",
            "2c4c2e343a8f64c625ba02b2aa12c7f8",
            format!("{{\"shortcode\":\"{}\"}}", post_id)
        );

        let response = self
            .client
            .get(&api_url)
            .send()
            .await
            .map_err(|e| BotError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(BotError::ApiError(format!(
                "Instagram API returned status: {}",
                response.status()
            )));
        }

        let data: serde_json::Value = response.json().await.map_err(|e| BotError::ParseError(e.to_string()))?;

        self.parse_media_response(data)
    }

    fn parse_media_response(&self, data: serde_json::Value) -> Result<MediaInfo, BotError> {
        let media = data
            .get("data")
            .and_then(|d| d.get("shortcode_media"))
            .ok_or_else(|| BotError::ParseError("Invalid response structure".into()))?;

        let typename = media
            .get("__typename")
            .and_then(|t| t.as_str())
            .ok_or_else(|| BotError::ParseError("Missing typename".into()))?;

        match typename {
            "GraphImage" => self.parse_image(media),
            "GraphVideo" => self.parse_video(media),
            "GraphSidecar" => self.parse_carousel(media),
            _ => Err(BotError::UnsupportedMedia(format!(
                "Unsupported media type: {}",
                typename
            ))),
        }
    }

    fn parse_image(&self, media: &serde_json::Value) -> Result<MediaInfo, BotError> {
        let url = media
            .get("display_url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| BotError::ParseError("Missing image URL".into()))?
            .to_string();

        // Estimate file size based on resolution
        let estimated_size = self.estimate_file_size(media)?;

        Ok(MediaInfo {
            url,
            media_type: MediaType::Image,
            file_size: estimated_size,
            carousel_items: vec![],
        })
    }

    fn parse_video(&self, media: &serde_json::Value) -> Result<MediaInfo, BotError> {
        let url = media
            .get("video_url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| BotError::ParseError("Missing video URL".into()))?
            .to_string();

        let file_size = media
            .get("video_duration")
            .and_then(|d| d.as_f64())
            .map(|duration| (duration * 1_000_000.0) as u64) // Rough estimate: 1MB per second
            .unwrap_or(0);

        Ok(MediaInfo {
            url,
            media_type: MediaType::Video,
            file_size,
            carousel_items: vec![],
        })
    }

    fn parse_carousel(&self, media: &serde_json::Value) -> Result<MediaInfo, BotError> {
        let edges = media
            .get("edge_sidecar_to_children")
            .and_then(|e| e.get("edges"))
            .and_then(|e| e.as_array())
            .ok_or_else(|| BotError::ParseError("Missing carousel edges".into()))?;

        let mut carousel_items = Vec::new();
        for edge in edges {
            let node = edge
                .get("node")
                .ok_or_else(|| BotError::ParseError("Missing node".into()))?;
            let is_video = node.get("is_video").and_then(|v| v.as_bool()).unwrap_or(false);

            let item = if is_video {
                self.parse_carousel_video(node)?
            } else {
                self.parse_carousel_image(node)?
            };

            carousel_items.push(item);
        }

        Ok(MediaInfo {
            url: "".to_string(), // Carousel doesn't have a single URL
            media_type: MediaType::Carousel,
            file_size: carousel_items.iter().map(|item| item.file_size).sum(),
            carousel_items,
        })
    }

    fn parse_carousel_image(&self, node: &serde_json::Value) -> Result<CarouselItem, BotError> {
        let url = node
            .get("display_url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| BotError::ParseError("Missing carousel image URL".into()))?
            .to_string();

        let estimated_size = self.estimate_file_size(node)?;

        Ok(CarouselItem {
            url,
            media_type: MediaType::Image,
            file_size: estimated_size,
        })
    }

    fn parse_carousel_video(&self, node: &serde_json::Value) -> Result<CarouselItem, BotError> {
        let url = node
            .get("video_url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| BotError::ParseError("Missing carousel video URL".into()))?
            .to_string();

        let file_size = node
            .get("video_duration")
            .and_then(|d| d.as_f64())
            .map(|duration| (duration * 1_000_000.0) as u64)
            .unwrap_or(0);

        Ok(CarouselItem {
            url,
            media_type: MediaType::Video,
            file_size,
        })
    }

    fn extract_post_id(&self, url: &Url) -> Result<String, BotError> {
        let path_segments: Vec<_> = url
            .path_segments()
            .ok_or_else(|| BotError::InvalidUrl("No path segments found".into()))?
            .collect();

        info!("Path segments: {:?}", path_segments);

        match path_segments.as_slice() {
            ["stories", _, post_id] | ["reel", post_id] | ["p", post_id] => Ok(post_id.to_string()),
            _ => Err(BotError::InvalidUrl("Invalid Instagram post URL format".into())),
        }
    }

    fn estimate_file_size(&self, media: &serde_json::Value) -> Result<u64, BotError> {
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

        // Rough estimate: 4 bytes per pixel + overhead
        Ok(width * height * 4 + 1024)
    }
}
