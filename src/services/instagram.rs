use crate::utils::error::BotError;
use anyhow::Result;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
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

#[derive(Clone)]
pub struct InstagramService {
    client: Client,
    api_endpoint: String,
    doc_id: String,
}

impl InstagramService {
    pub fn new(client: Client, api_endpoint: String, doc_id: String) -> Self {
        Self {
            client,
            api_endpoint,
            doc_id,
        }
    }

    pub async fn get_media_info(&self, url: &str) -> Result<MediaInfo, BotError> {
        info!("Parsing URL: {}", url);
        let parsed_url = Url::parse(url).map_err(|_| BotError::InvalidUrl("Invalid Instagram URL".into()))?;

        info!("Extracting shortcode from URL...");
        let shortcode = self.extract_shortcode(&parsed_url)?;

        info!("Shortcode: {}", shortcode);

        info!("Fetching media info from Instagram's API...");
        let media_info = self.fetch_media_info(&shortcode).await?;

        Ok(media_info)
    }

    async fn fetch_media_info(&self, shortcode: &str) -> Result<MediaInfo, BotError> {
        let api_url = self.api_endpoint.clone();

        let body = serde_json::json!({
            "doc_id": self.doc_id,
            "variables": {
                "shortcode": shortcode
            }
        });

        info!("Making request to: {} with body: {:?}", api_url, body);

        let response = self
            .client
            .post(&api_url)
            .header(header::ACCEPT, "*/*")
            .header(header::CONTENT_TYPE, "application/json")
            // .header(header::ACCEPT_ENCODING, "gzip, deflate, br")
            .header(header::HOST, "www.instagram.com")
            .json(&body)
            .send()
            .await
            .map_err(|e| BotError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(BotError::ApiError(format!(
                "Instagram API returned status: {}",
                response.status()
            )));
        }

        info!("Response status: {}", response.status());

        // Let reqwest handle decompression automatically
        let data: serde_json::Value = response.json().await.map_err(|e| {
            error!("JSON parse error: {:?}", e);
            BotError::ParseError(e.to_string())
        })?;

        info!("Parsed JSON: {:?}", data);

        self.parse_media_response(data)
    }

    // TODO: test
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

    fn extract_shortcode(&self, url: &Url) -> Result<String, BotError> {
        let path_segments: Vec<_> = url
            .path_segments()
            .ok_or_else(|| BotError::InvalidUrl("No path segments found".into()))?
            .collect();

        info!("Path segments: {:?}", path_segments);

        match path_segments.as_slice() {
            ["stories", _, shortcode] | ["reel", shortcode, _] | ["p", shortcode, _] => Ok(shortcode.to_string()),
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
