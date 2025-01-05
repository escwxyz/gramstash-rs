use std::sync::Arc;

use reqwest::{cookie::Jar, Client};
use url::Url;

use crate::utils::{
    error::{BotError, BotResult},
    http,
};

use super::{types::InstagramIdentifier, SessionData, TwoFactorAuthPending};

#[derive(Clone)]
pub struct InstagramService {
    pub client: Client,
    pub cookie_jar: Arc<Jar>,
    pub session_data: SessionData,
    pub two_factor_auth_pending: Option<TwoFactorAuthPending>,
}

impl InstagramService {
    pub fn new() -> Self {
        info!("Initializing InstagramService");
        let cookie_jar = Arc::new(Jar::default());
        let client = http::create_instagram_client(Arc::clone(&cookie_jar));
        info!("HTTP client initialized");

        // Initialize empty session data
        let session_data = SessionData {
            cookies: Vec::new(),
            user_id: None,
            username: None,
            csrf_token: None,
            session_id: None,
            device_id: None,
            machine_id: None,
            rur: None,
        };

        Self {
            client,
            cookie_jar,
            session_data,
            two_factor_auth_pending: None,
        }
    }
    pub fn parse_instagram_url(&self, url: &Url) -> BotResult<InstagramIdentifier> {
        let path_segments: Vec<_> = url
            .path_segments()
            .ok_or_else(|| BotError::InvalidUrl("No path segments found".into()))?
            .collect();

        info!("Parsing Instagram URL with path segments: {:?}", path_segments);

        match path_segments.as_slice() {
            ["stories", username, story_id] => Ok(InstagramIdentifier::Story {
                username: username.to_string(),
                shortcode: story_id.to_string(),
            }),
            ["p", shortcode, ..] => Ok(InstagramIdentifier::Post {
                shortcode: shortcode.to_string(),
            }),
            ["reel", shortcode, ..] => Ok(InstagramIdentifier::Reel {
                shortcode: shortcode.to_string(),
            }),
            _ => Err(BotError::InvalidUrl("Invalid Instagram URL format".into())),
        }
    }
}
