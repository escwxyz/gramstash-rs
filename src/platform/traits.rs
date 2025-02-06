use std::any::Any;

use async_trait::async_trait;
use teloxide::{adaptors::Throttle, types::ChatId, Bot};

use crate::error::HandlerResult;

use super::{MediaFile, MediaInfo, Platform, PlatformError, PlatformIdentifier};

pub trait IntoMediaInfo {
    fn into_media_info(self) -> Result<MediaInfo, PlatformError>;
}

#[async_trait]
pub trait PlatformCapability: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;

    fn platform_id(&self) -> Platform;
    fn platform_name(&self) -> &str;

    async fn parse_url(&self, url_str: &str) -> Result<PlatformIdentifier, PlatformError>;

    async fn fetch_resource(&self, identifier: &PlatformIdentifier) -> HandlerResult<MediaInfo>;

    // 发送媒体到 Telegram，在此之前可以缓存或者上传媒体
    async fn send_to_telegram(&self, bot: &Throttle<Bot>, chat_id: ChatId, media_file: &MediaFile)
        -> HandlerResult<()>;
}
