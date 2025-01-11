use crate::{
    error::{BotError, BotResult, InstagramError, ServiceError},
    state::AppState,
};

use super::{
    types::{ContentType, MediaAuthor, MediaContent, PostContent},
    CarouselItem, InstagramService, MediaInfo,
};
use anyhow::{anyhow, Context, Result};
use url::Url;

impl InstagramService {
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
            .context("Failed to fetch from instagram API")?;

        if !response.status().is_success() {
            return Err(BotError::ServiceError(ServiceError::InstagramError(
                InstagramError::ApiError(format!("Instagram API returned status: {}", response.status())),
            )));
        }

        let data: serde_json::Value = response.json().await.context("Failed to parse JSON")?;

        self.parse_media_response(data)
    }

    fn parse_media_response(&self, data: serde_json::Value) -> BotResult<MediaInfo> {
        let media = data
            .get("data")
            .and_then(|d| d.get("xdt_shortcode_media"))
            .ok_or_else(|| anyhow!("Invalid response structure"))?; // TODO

        let typename = media
            .get("__typename")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow!("Missing typename"))?; // TODO

        match typename {
            "XDTGraphImage" => self.parse_image(media),
            "XDTGraphVideo" => self.parse_reel(media),
            "XDTGraphSidecar" => self.parse_carousel(media),
            _ => Err(BotError::InvalidUrl(format!("Unspported media type: {}", typename))),
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

    fn parse_image(&self, media: &serde_json::Value) -> BotResult<MediaInfo> {
        let url = self.find_display_url(media)?;

        Ok(MediaInfo {
            content: MediaContent::Post(PostContent::Single {
                url: Url::parse(&url).map_err(|e| BotError::InvalidUrl(e.to_string()))?,
                content_type: ContentType::Image,
                // caption: None,
                // timestamp: None,
            }),
            author: self.get_author(media)?,
        })
    }

    fn get_author(&self, media: &serde_json::Value) -> Result<MediaAuthor> {
        let username = media
            .get("owner")
            .and_then(|o| o.get("username"))
            .and_then(|u| u.as_str())
            .unwrap_or("unknown");

        Ok(MediaAuthor {
            username: username.to_string(),
        })
    }

    fn find_display_url(&self, media_or_node: &serde_json::Value) -> Result<String> {
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

    // For reels
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

    fn parse_carousel_image(&self, node: &serde_json::Value) -> Result<CarouselItem> {
        let url = self.find_display_url(node)?;
        Ok(CarouselItem {
            url: Url::parse(&url).map_err(|e| BotError::InvalidUrl(e.to_string()))?,
            content_type: ContentType::Image,
            // caption: None,
            // timestamp: None,
        })
    }

    fn parse_carousel_video(&self, node: &serde_json::Value) -> Result<CarouselItem> {
        let url = node
            .get("video_url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| anyhow!("Missing carousel video URL"))?
            .to_string();
        Ok(CarouselItem {
            url: Url::parse(&url).map_err(|e| BotError::InvalidUrl(e.to_string()))?,
            content_type: ContentType::Reel,
            // caption: None,
        })
    }
}
