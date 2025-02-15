mod download;
mod error;
mod model;
pub mod traits;
mod util;

use std::sync::Arc;

use dashmap::DashMap;
use instagram::model::InstagramIdentifier;

pub use error::*;
pub use model::*;
use traits::PlatformCapability;
pub use util::*;

pub use instagram::PlatformInstagram;

pub mod instagram;

#[derive(Clone)]
pub struct PlatformRegistry {
    platforms: Arc<DashMap<Platform, Arc<dyn PlatformCapability>>>,
}

impl PlatformRegistry {
    pub fn new() -> Result<Self, PlatformError> {
        info!("Initializing platform registry");

        let platforms = Arc::new(DashMap::<Platform, Arc<dyn PlatformCapability>>::new());
        info!("Registering Instagram platform");
        platforms.insert(Platform::Instagram, Arc::new(PlatformInstagram::new()?));

        info!("Platform registry initialized");
        Ok(Self { platforms })
    }

    pub fn generate_identifier(&self, resource: &PlatformIdentifier) -> String {
        match resource {
            PlatformIdentifier::Instagram(InstagramIdentifier::Story { story_id, username: _ }) => {
                format!("instagram:{}", story_id)
            }
            PlatformIdentifier::Instagram(
                InstagramIdentifier::Post { shortcode } | InstagramIdentifier::Reel { shortcode },
            ) => {
                format!("instagram:{}", shortcode)
            }
        }
    }

    pub fn get_platform<T: PlatformCapability + 'static>(&self, platform: &Platform) -> Option<Arc<T>> {
        self.platforms.get(platform).and_then(|p| {
            let platform_ref = p.value();

            if platform_ref.as_any().is::<T>() {
                Some(platform_ref.clone()).map(|arc| unsafe { Arc::from_raw(Arc::into_raw(arc) as *const T) })
            } else {
                error!("Failed to get platform: wrong type");
                None
            }
        })
    }

    pub async fn get_supported_platforms(&self) -> Vec<Platform> {
        self.platforms
            .iter()
            .map(|platform| platform.key().clone())
            .collect::<Vec<_>>()
    }
}
