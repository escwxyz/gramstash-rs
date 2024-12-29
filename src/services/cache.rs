use crate::{services::instagram::MediaInfo, state::AppState};
use anyhow::Result;
use redis::AsyncCommands;
pub struct CacheService;

impl CacheService {
    pub async fn get_media_info(shortcode: &str) -> Result<Option<MediaInfo>> {
        let mut conn = AppState::get().redis.get_connection().await?;
        let key = Self::generate_key(shortcode);

        let data: Option<String> = conn.get(&key).await?;

        info!("Cache hit: {}", data.is_some());

        match data {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }

    pub async fn set_media_info(shortcode: &str, media_info: &MediaInfo) -> Result<()> {
        let key = Self::generate_key(shortcode);

        let json = serde_json::to_string(media_info)?;

        info!("Setting cache for key: {}", key);

        AppState::get()
            .redis
            .set_cached(&key, &json, AppState::get().config.cache.expiry_secs)
            .await?;

        Ok(())
    }

    fn generate_key(shortcode: &str) -> String {
        format!("media_cache:{}", shortcode)
    }
}
