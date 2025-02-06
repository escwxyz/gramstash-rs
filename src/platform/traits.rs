use std::any::Any;

use async_trait::async_trait;
use teloxide::{adaptors::Throttle, types::ChatId, Bot};

use crate::error::HandlerResult;

use super::{MediaFile, Platform, PlatformError, PlatformIdentifier};

#[async_trait]
pub trait PlatformCapability: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;

    fn platform_id(&self) -> Platform;
    #[allow(unused)]
    fn platform_name(&self) -> &str;

    async fn parse_url(&self, url_str: &str) -> Result<PlatformIdentifier, PlatformError>;

    async fn fetch_resource(&self, identifier: &PlatformIdentifier) -> HandlerResult<MediaFile>;

    #[allow(unused)]
    async fn pre_process(
        &self,
        bot: &Throttle<Bot>,
        chat_id: ChatId,
        media_info: &MediaFile,
    ) -> HandlerResult<MediaFile>;

    async fn send_to_telegram(&self, bot: &Throttle<Bot>, chat_id: ChatId, media_file: &MediaFile)
        -> HandlerResult<()>;

    #[allow(unused)]
    async fn post_process(&self, bot: &Throttle<Bot>, chat_id: ChatId, media_info: &MediaFile) -> HandlerResult<()>;
}
