use reqwest::{
    cookie::{CookieStore, Jar},
    Client,
};
use std::{sync::Arc, time::Duration};
use url::Url;

use super::types::{SerializableCookie, SessionData};
use crate::{error::BotResult, utils::http};

#[derive(Clone)]
pub struct AuthClient {
    pub client: reqwest::Client,
    pub cookie_jar: Arc<Jar>,
}

impl AuthClient {
    pub fn new() -> BotResult<Self> {
        let cookie_jar = Arc::new(Jar::default());
        let client = Self::create_client(Arc::clone(&cookie_jar))?;

        Ok(Self { client, cookie_jar })
    }

    fn create_client(cookie_store: Arc<Jar>) -> BotResult<Client> {
        let builder = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(30))
            .cookie_provider(Arc::clone(&cookie_store))
            .default_headers(http::build_desktop_instagram_headers())
            .user_agent(http::INSTAGRAM_USER_AGENT);

        http::build_client(builder)
    }

    pub fn extract_cookies(&self) -> Vec<SerializableCookie> {
        let base_url = Url::parse("https://www.instagram.com").unwrap();

        let cookies = self
            .cookie_jar
            .cookies(&base_url)
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
    #[allow(dead_code)]
    pub fn restore_cookies(&mut self, session_data: SessionData) -> BotResult<()> {
        // Clear existing session
        self.cookie_jar = Arc::new(Jar::default());

        // Add all cookies from session data
        for cookie in session_data.cookies {
            let cookie_str = format!(
                "{}={}; Domain={}; Path={}",
                cookie.name, cookie.value, cookie.domain, cookie.path
            );
            self.cookie_jar
                .add_cookie_str(&cookie_str, &"https://www.instagram.com".parse().unwrap());
        }

        Ok(())
    }
}
