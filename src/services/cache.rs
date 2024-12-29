use crate::{services::instagram::MediaInfo, state::AppState};
use anyhow::Result;
use redis::AsyncCommands;
use serde_json;

pub struct CacheService;

impl CacheService {
    pub async fn get_media_info(shortcode: &str) -> Result<Option<MediaInfo>> {
        let mut conn = AppState::get().redis.get_connection().await?;
        let key = Self::generate_key(shortcode);

        let data: Option<String> = conn.get(&key).await?;

        match data {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }

    pub async fn set_media_info(shortcode: &str, media_info: &MediaInfo) -> Result<()> {
        let expiry_secs = AppState::get().config.cache.expiry_secs;
        let mut conn = AppState::get().redis.get_connection().await?;
        let key = Self::generate_key(shortcode);

        let json = serde_json::to_string(media_info)?;
        conn.set_ex::<_, _, u64>(&key, json, expiry_secs).await?;

        Ok(())
    }

    fn generate_key(shortcode: &str) -> String {
        format!("media_cache:{}", shortcode)
    }
}
