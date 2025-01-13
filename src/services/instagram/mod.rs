mod auth;
mod post;
mod story;
pub(crate) mod types;

use crate::{
    error::{BotError, BotResult},
    utils::http,
};
use reqwest::{
    cookie::{CookieStore, Jar},
    Client,
};
use std::sync::Arc;
pub use types::{CarouselItem, MediaInfo};
use types::{InstagramIdentifier, MediaAuthor};
use url::Url;

use super::session::{SerializableCookie, SessionData};

#[derive(Clone)]
pub struct InstagramService {
    pub public_client: Client,
    pub auth_client: Client,
    pub cookie_jar: Arc<Jar>,
}

impl InstagramService {
    pub fn new() -> BotResult<Self> {
        let cookie_jar = Arc::new(Jar::default());
        let public_client = http::create_instagram_public_client()?;
        let auth_client = http::create_instagram_auth_client(Arc::clone(&cookie_jar))?;
        Ok(Self {
            public_client,
            auth_client,
            cookie_jar,
        })
    }

    // Just call this function when we get a new session after initializing the service
    pub fn restore_cookies(&mut self, session_data: SessionData) -> BotResult<()> {
        // Clear existing session
        self.cookie_jar = Arc::new(Jar::default());
        // Restore cookies from session data
        for cookie in session_data.cookies {
            self.cookie_jar.add_cookie_str(
                &format!("{}={}", cookie.name, cookie.value),
                &"https://www.instagram.com".parse().unwrap(),
            );
        }
        Ok(())
    }

    fn extract_cookies(&self) -> Vec<SerializableCookie> {
        let cookies = self
            .cookie_jar
            .cookies(&"https://www.instagram.com".parse().unwrap())
            .map(|c| {
                let cookie_str = c.to_str().unwrap();
                let parts: Vec<&str> = cookie_str.split(';').collect();
                let name_value: Vec<&str> = parts[0].split('=').collect();

                SerializableCookie {
                    name: name_value[0].to_string(),
                    value: name_value[1].to_string(),
                    domain: ".instagram.com".to_string(),
                    path: "/".to_string(),
                }
            })
            .into_iter()
            .collect();

        cookies
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

    fn get_author(&self, media: &serde_json::Value) -> BotResult<MediaAuthor> {
        let username = media
            .get("owner")
            .and_then(|o| o.get("username"))
            .and_then(|u| u.as_str())
            .unwrap_or("unknown");

        Ok(MediaAuthor {
            username: username.to_string(),
        })
    }
}
