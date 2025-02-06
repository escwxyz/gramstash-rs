use crate::{error::BotError, service::Cacheable, state::AppState};

use super::{traits::IntoMediaInfo, DownloadState, Platform, PlatformCapability, PlatformError, PlatformRegistry};

impl PlatformRegistry {
    pub async fn handle_download<P: PlatformCapability + 'static, C: Cacheable + IntoMediaInfo + 'static>(
        &self,
        platform: &Platform,
        url: &str,
        telegram_user_id: &str,
    ) -> Result<DownloadState, BotError> {
        info!("handle_download");
        let platform_service = self
            .get_platform::<P>(platform)
            .ok_or_else(|| PlatformError::ResourceError("Platform not found".into()))?;
        let resource = platform_service.parse_url(url).await?;
        info!("resource: {:?}", resource);
        let identifier = self.generate_identifier(&resource);
        info!("identifier: {:?}", identifier);

        let ratelimit = AppState::get()?.service_registry.ratelimit;
        info!("checking rate limit");
        if !ratelimit.check_rate_limit(telegram_user_id, &identifier).await? {
            info!("rate limited");
            return Ok(DownloadState::RateLimited);
        }

        let cache_service = AppState::get()?.service_registry.cache;
        info!("checking cache");
        if let Some(cached) = cache_service.get::<C>(&identifier).await? {
            info!("cache hit");
            return Ok(DownloadState::Success(cached.into_media_info()?));
        }

        info!("fetching resource");
        match platform_service.fetch_resource(&resource).await {
            Ok(media_info) => {
                info!("resource fetched");
                Ok(DownloadState::Success(media_info))
            }
            Err(_) => {
                info!("resource fetch failed");
                Ok(DownloadState::Error)
            }
        }
    }
}
