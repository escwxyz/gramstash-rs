use crate::{error::BotError, platform::MediaFile, state::AppState};

use super::{DownloadState, Platform, PlatformCapability, PlatformError, PlatformRegistry};

impl PlatformRegistry {
    pub async fn handle_download<P: PlatformCapability + 'static>(
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
        let identifier = self.generate_identifier(&resource); // <platform>:<identifier>
        info!("identifier: {:?}", identifier);

        let ratelimit = AppState::get()?.service_registry.ratelimit;
        info!("checking rate limit");
        if !ratelimit.check_rate_limit(telegram_user_id, &identifier).await? {
            info!("rate limited");
            return Ok(DownloadState::RateLimited);
        }

        let cache_service = AppState::get()?.service_registry.cache;

        if let Some(cached) = cache_service.get::<MediaFile>(&identifier).await? {
            info!("cache hit");
            return Ok(DownloadState::Success(cached));
        }

        info!("cache missed");

        info!("fetching resource");
        match platform_service.fetch_resource(&resource).await {
            Ok(media_file) => {
                info!("resource fetched");
                Ok(DownloadState::Success(media_file))
            }
            Err(e) => {
                info!("resource fetch failed: {:?}", e);
                Ok(DownloadState::Error)
            }
        }
    }
}
