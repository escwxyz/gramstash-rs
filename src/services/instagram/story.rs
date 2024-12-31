use anyhow::{anyhow, Context, Result};

use super::{InstagramService, MediaInfo, MediaType};

impl InstagramService {
    pub async fn get_story_info(self, story_id: &str) -> Result<MediaInfo> {
        if !self.is_logged_in() {
            return Err(anyhow!("Please login first using /login command to access stories"));
        }

        // TODO
        let api_url = "https://www.instagram.com/api/v1/feed/reels_media/";

        // TODO: test this
        let body = serde_json::json!({
            "reel_ids": [story_id],    // Array of story IDs
            "source": "story_viewer_reels"
        });

        info!("Sending story request with body: {:?}", body);

        let response = self
            .client
            .post(api_url)
            .json(&body)
            .send()
            .await
            .context("Failed to fetch story from Instagram API")?;

        if !response.status().is_success() {
            return Err(anyhow!("Instagram API returned status: {}", response.status()));
        }

        let data: serde_json::Value = response.json().await.context("Failed to parse JSON")?;

        info!("Story API response: {:?}", data);

        self.parse_story_response(data, story_id)
    }

    fn parse_story_response(&self, data: serde_json::Value, story_id: &str) -> Result<MediaInfo> {
        let reels = data
            .get("reels")
            .and_then(|r| r.get(story_id))
            .ok_or_else(|| anyhow!("Invalid story response structure"))?;

        let items = reels
            .get("items")
            .and_then(|i| i.as_array())
            .ok_or_else(|| anyhow!("No items found in story"))?;

        // Usually stories have one item, but let's get the first one
        let item = items.first().ok_or_else(|| anyhow!("Story has no items"))?;

        // Determine media type and get URL
        let is_video = item
            .get("media_type")
            .and_then(|t| t.as_u64())
            .map(|t| t == 2) // 2 = video, 1 = image
            .unwrap_or(false);

        if is_video {
            let url = item
                .get("video_versions")
                .and_then(|v| v.as_array())
                .and_then(|v| v.first())
                .and_then(|v| v.get("url"))
                .and_then(|u| u.as_str())
                .ok_or_else(|| anyhow!("Missing video URL"))?
                .to_string();

            Ok(MediaInfo {
                url,
                media_type: MediaType::Video,
                carousel_items: vec![],
            })
        } else {
            let url = item
                .get("image_versions2")
                .and_then(|i| i.get("candidates"))
                .and_then(|c| c.as_array())
                .and_then(|c| c.first())
                .and_then(|c| c.get("url"))
                .and_then(|u| u.as_str())
                .ok_or_else(|| anyhow!("Missing image URL"))?
                .to_string();

            Ok(MediaInfo {
                url,
                media_type: MediaType::Image,
                carousel_items: vec![],
            })
        }
    }
}
