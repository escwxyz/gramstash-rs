use crate::{
    config::AppConfig,
    error::{BotError, BotResult, ServiceError},
    state::AppState,
};
use redis::AsyncCommands;
use serde_json;

use super::instagram::InstagramMedia;

pub struct CacheService;

impl CacheService {
    /// Generates a cache key in the format "media:<telegram_user_id>:<instagram_username>:<identifier>"
    fn generate_key(telegram_user_id: &str, instagram_username: &str, identifier: &str) -> String {
        format!("media:{}:{}:{}", telegram_user_id, instagram_username, identifier)
    }

    /// Get media from cache
    pub async fn get_media_from_redis(telegram_user_id: &str, identifier: &str) -> BotResult<Option<InstagramMedia>> {
        let mut conn = AppState::get()?.redis.get_connection().await?;

        let keys: Vec<String> = conn
            .keys(&format!("media:{}:*:{}", telegram_user_id, identifier))
            .await?;

        if keys.is_empty() {
            return Ok(None);
        }

        let key = keys
            .first()
            .ok_or(BotError::ServiceError(ServiceError::Cache("No key found".to_string())))?;

        let data: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| BotError::ServiceError(ServiceError::Cache(e.to_string())))?;

        match data {
            Some(json) => {
                Ok(Some(serde_json::from_str(&json).map_err(|e| {
                    BotError::ServiceError(ServiceError::Cache(e.to_string()))
                })?))
            }
            None => Ok(None),
        }
    }

    /// Set media in cache
    pub async fn cache_media_to_redis(
        telegram_user_id: &str,
        instagram_username: &str,
        identifier: &str,
        media: &InstagramMedia,
    ) -> BotResult<()> {
        let mut conn = AppState::get()?.redis.get_connection().await?;
        let key = Self::generate_key(telegram_user_id, instagram_username, identifier);
        let expiry_secs = AppConfig::get()?.cache.expiry_secs;

        let json =
            serde_json::to_string(media).map_err(|e| BotError::ServiceError(ServiceError::Cache(e.to_string())))?;

        conn.set_ex::<_, _, String>(&key, json, expiry_secs)
            .await
            .map_err(|e| BotError::ServiceError(ServiceError::Cache(e.to_string())))?;

        Ok(())
    }
}
