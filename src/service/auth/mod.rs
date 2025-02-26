mod error;
mod model;
pub use error::AuthError;
pub use model::*;

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use chrono::Utc;
use reqwest::cookie::CookieStore;
use url::Url;

use crate::platform::{traits::PlatformCapability, Platform, PlatformInstagram, PlatformRegistry};

use super::{
    http::{HttpClient, HttpService},
    session::{Session, SessionData},
};

#[async_trait]
pub trait PlatformAuth: PlatformCapability {
    fn get_http_service(&self) -> HttpService;
    async fn login(&self, credentials: &Credentials) -> Result<SessionData, AuthError>;
    // async fn logout(&self) -> Result<(), AuthError>;
    async fn verify_session(&self, session: &Session) -> Result<bool, AuthError>;
    async fn get_csrf_token(&self) -> Result<String, AuthError> {
        let jar = self.get_http_service().get_cookie_jar();

        let platform = self.platform_id();

        let host = match platform {
            Platform::Instagram => "https://www.instagram.com",
            _ => return Err(AuthError::Other("Platform not supported".into())),
        };

        let cookies = jar.cookies(&host.parse().unwrap());

        if let Some(cookies) = cookies {
            let cookie_str = cookies.to_str().unwrap();
            for cookie in cookie_str.split(';').map(|s| s.trim()) {
                let parts: Vec<&str> = cookie.split('=').collect();
                if parts.len() == 2 && parts[0] == "csrftoken" {
                    return Ok(parts[1].to_string());
                }
            }
        }

        Err(AuthError::CookieNotFound)
    }

    fn extract_cookie(&self, name: &str, expires: Duration) -> Result<CookieData, AuthError> {
        let platform = self.platform_id();
        let (host, domain) = match platform {
            Platform::Instagram => ("https://www.instagram.com", ".instagram.com"),
            _ => return Err(AuthError::Other("Platform not supported".into())),
        };
        let base_url = Url::parse(host).unwrap();
        let http_service = self.get_http_service();
        let jar = http_service.get_cookie_jar();

        if let Some(cookies) = jar.cookies(&base_url) {
            let cookie_str = cookies.to_str().unwrap();
            for cookie in cookie_str.split(';').map(|s| s.trim()) {
                let parts: Vec<&str> = cookie.split('=').collect();
                if parts.len() == 2 && parts[0] == name {
                    return Ok(CookieData {
                        name: name.to_string(),
                        value: parts[1].to_string(),
                        domain: domain.to_string(),
                        path: "/".to_string(),
                        expires: Some(Utc::now() + expires),
                    });
                }
            }
        }

        Err(AuthError::CookieNotFound)
    }
}

#[derive(Clone)]
pub struct AuthService {
    platform_registry: Arc<PlatformRegistry>,
}

impl AuthService {
    pub fn new(platform_registry: Arc<PlatformRegistry>) -> Self {
        Self { platform_registry }
    }

    // fn create_client(cookie_store: Arc<Jar>, user_agent: &str) -> Result<reqwest::Client, AuthError> {
    //     let builder = reqwest::Client::builder()
    //         .timeout(Duration::from_secs(30))
    //         .connect_timeout(Duration::from_secs(30))
    //         .cookie_provider(Arc::clone(&cookie_store))
    //         .default_headers(http::build_desktop_instagram_headers(false))
    //         .user_agent(user_agent);

    //     http::build_client(builder).map_err(|e| AuthError::Other(e.to_string()))
    // }

    pub async fn login(&self, credentials: &Credentials) -> Result<SessionData, AuthError> {
        let platform = self
            .platform_registry
            .get_platform::<PlatformInstagram>(&credentials.platform)
            .ok_or_else(|| AuthError::Other("Platform not supported".into()))?;

        let auth = platform
            .as_any()
            .downcast_ref::<Box<dyn PlatformAuth>>()
            .ok_or_else(|| AuthError::Other("Platform does not support auth".into()))?;

        auth.login(credentials).await
    }

    pub async fn verify_session(&self, session: &Session) -> Result<bool, AuthError> {
        let platform = self
            .platform_registry
            .get_platform::<PlatformInstagram>(&session.platform)
            .ok_or_else(|| AuthError::Other("Platform not supported".into()))?;

        let auth = platform
            .as_any()
            .downcast_ref::<Box<dyn PlatformAuth>>()
            .ok_or_else(|| AuthError::Other("Platform does not support auth".into()))?;

        auth.verify_session(session).await
    }

    // pub async fn logout(&self, platform: &Platform) -> Result<(), AuthError> {
    //     let platform = self
    //         .platform_registry
    //         .get_platform::<PlatformInstagram>(platform)
    //         .ok_or_else(|| AuthError::Other("Platform not supported".into()))?;

    //     let auth = platform
    //         .as_any()
    //         .downcast_ref::<Box<dyn PlatformAuth>>()
    //         .ok_or_else(|| AuthError::Other("Platform does not support auth".into()))?;

    //     auth.logout().await
    // }
}
