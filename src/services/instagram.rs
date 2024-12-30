use crate::utils::http;
use anyhow::{anyhow, Context, Result};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaType {
    Image,
    Video,
    Carousel,
}

// TODO: improve this struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub url: String,
    pub media_type: MediaType,
    pub carousel_items: Vec<CarouselItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarouselItem {
    pub url: String,
    pub media_type: MediaType,
}

#[derive(Clone)]
pub struct InstagramService {
    pub client: Client,
    pub api_endpoint: String,
    pub doc_id: String,
}

impl InstagramService {
    pub fn new(api_endpoint: String, doc_id: String) -> Self {
        info!("Initializing InstagramService");
        let client = http::create_download_client();
        info!("HTTP client initialized");

        Self {
            client,
            api_endpoint,
            doc_id,
        }
    }

    pub async fn get_media_info(&self, shortcode: &str) -> Result<MediaInfo> {
        Ok(self.fetch_media_info(&shortcode).await?)
    }

    async fn fetch_media_info(&self, shortcode: &str) -> Result<MediaInfo> {
        let api_url = self.api_endpoint.clone();

        let body = serde_json::json!({
            "doc_id": self.doc_id,
            "variables": {
                "shortcode": shortcode
            }
        });

        let response = self
            .client
            .post(&api_url)
            .header(header::ACCEPT, "*/*")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::HOST, "www.instagram.com")
            .json(&body)
            .send()
            .await
            .context("Failed to fetch from instagram API")?;

        if !response.status().is_success() {
            return Err(anyhow!("Instagram API returned status: {}", response.status()));
        }

        let data: serde_json::Value = response.json().await.context("Failed to parse JSON")?;

        self.parse_media_response(data)
    }

    // TODO
    fn parse_media_response(&self, data: serde_json::Value) -> Result<MediaInfo> {
        let media = data
            .get("data")
            .and_then(|d| d.get("xdt_shortcode_media"))
            .ok_or_else(|| anyhow!("Invalid response structure"))?;

        info!("Media: {:?}", media);
        let typename = media
            .get("__typename")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow!("Missing typename"))?;

        info!("Typename: {:?}", typename);

        match typename {
            "XDTGraphImage" => self.parse_image(media),
            "XDTGraphVideo" => self.parse_video(media),
            "XDTGraphSidecar" => self.parse_carousel(media),
            // TODO: support stories
            _ => Err(anyhow!("Unspported media type: {}", typename)),
        }
    }

    fn get_dimensions(&self, media: &serde_json::Value) -> Result<(u64, u64)> {
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

    // For single image post
    fn parse_image(&self, media: &serde_json::Value) -> Result<MediaInfo> {
        let url = self.find_display_url(media)?;

        Ok(MediaInfo {
            url,
            media_type: MediaType::Image,
            carousel_items: vec![],
        })
    }

    fn find_display_url(&self, media_or_node: &serde_json::Value) -> Result<String> {
        let (width, height) = self.get_dimensions(media_or_node)?;

        // Find the display resource that matches original dimensions
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
                    .or_else(|| resources.last()) // Fallback to highest resolution if no exact match
                    .and_then(|resource| resource.get("src"))
                    .and_then(|u| u.as_str())
            })
            .unwrap_or_else(|| {
                // Fallback to display_url if display_resources fails
                media_or_node.get("display_url").and_then(|u| u.as_str()).unwrap_or("")
            })
            .to_string();

        Ok(url)
    }

    // For reels
    fn parse_video(&self, media: &serde_json::Value) -> Result<MediaInfo> {
        let url = media
            .get("video_url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| anyhow!("Missing video URL"))?
            .to_string();

        Ok(MediaInfo {
            url,
            media_type: MediaType::Video,
            carousel_items: vec![],
        })
    }

    fn parse_carousel(&self, media: &serde_json::Value) -> Result<MediaInfo> {
        let edges = media
            .get("edge_sidecar_to_children")
            .and_then(|e| e.get("edges"))
            .and_then(|e| e.as_array())
            .ok_or_else(|| anyhow!("Missing carousel edges"))?;

        let mut carousel_items = Vec::new();
        for edge in edges {
            let node = edge.get("node").ok_or_else(|| anyhow!("Missing node"))?;
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
            carousel_items,
        })
    }

    fn parse_carousel_image(&self, node: &serde_json::Value) -> Result<CarouselItem> {
        let url = self.find_display_url(node)?;
        Ok(CarouselItem {
            url,
            media_type: MediaType::Image,
        })
    }

    fn parse_carousel_video(&self, node: &serde_json::Value) -> Result<CarouselItem> {
        let url = node
            .get("video_url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| anyhow!("Missing carousel video URL"))?
            .to_string();
        Ok(CarouselItem {
            url,
            media_type: MediaType::Video,
        })
    }

    pub fn extract_shortcode(&self, url: &Url) -> Result<String> {
        let path_segments: Vec<_> = url
            .path_segments()
            .ok_or_else(|| anyhow!("No path segments found"))?
            .collect();

        info!("Path segments: {:?}", path_segments);

        match path_segments.as_slice() {
            // TODO: support more formats
            ["stories", _, shortcode] | ["reel", shortcode, _] | ["p", shortcode, _] => Ok(shortcode.to_string()),
            _ => Err(anyhow!("Invalid Instagram post URL format")),
        }
    }
}
