use std::{sync::Arc, time::Duration};

use chrono::Utc;
use reqwest::cookie::{CookieStore, Jar};
use serde::Deserialize;
use url::Url;

use crate::{
    error::{AuthenticationError, BotError, BotResult, InstagramError, ServiceError},
    state::AppState,
    utils::http,
};

use super::session::{InstagramCookie, SessionData};

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub status: String,
    #[allow(dead_code)]
    pub authenticated: Option<bool>,
    #[allow(dead_code)]
    pub user: Option<bool>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    pub message: Option<String>,
    pub two_factor_required: Option<bool>,
    pub checkpoint_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

pub struct AuthService {
    pub client: reqwest::Client,
    pub cookie_jar: Arc<Jar>,
}

impl AuthService {
    pub fn new() -> BotResult<Self> {
        let cookie_jar = Arc::new(Jar::default());
        let client = Self::create_client(Arc::clone(&cookie_jar))?;

        Ok(Self { client, cookie_jar })
    }

    pub fn restore_cookies(&mut self, session: &SessionData) -> BotResult<()> {
        info!("Restoring cookies...");
        self.cookie_jar = Arc::new(Jar::default());

        // Add each cookie from session data
        let cookies = [
            &session.sessionid,
            &session.ds_user_id,
            &session.csrftoken,
            &session.ig_did,
            &session.mid,
        ];

        for cookie in cookies {
            let cookie_str = format!(
                "{}={}; Domain={}; Path={}{}",
                cookie.name,
                cookie.value,
                cookie.domain,
                cookie.path,
                cookie
                    .expires
                    .map(|e| format!("; Expires={}", e.to_rfc3339()))
                    .unwrap_or_default()
            );
            self.cookie_jar
                .add_cookie_str(&cookie_str, &"https://www.instagram.com".parse().unwrap());
        }

        // Update client with new cookie jar

        self.client = Self::create_client(Arc::clone(&self.cookie_jar))?;

        Ok(())
    }

    pub async fn login(&self, credentials: Credentials) -> BotResult<SessionData> {
        // Step 1: Get initial cookies and CSRF token
        info!("Initializing login flow");
        self.client
            .get("https://www.instagram.com/")
            .send()
            .await
            .map_err(|e| BotError::ServiceError(ServiceError::InstagramError(InstagramError::NetworkError(e))))?;

        // Step 2: Wait for cookie processing
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Step 3: Get CSRF token from cookies
        let csrf_token = self.get_csrf_token().await?;
        info!("Got CSRF token: {}", csrf_token);

        // Step 4: Perform login with proper headers
        let login_response = self
            .perform_login(&credentials.username, &credentials.password, &csrf_token)
            .await?;

        // Step 5: Create session data with all necessary cookies
        let session_data = SessionData {
            // User information
            user_id: login_response.user_id.ok_or_else(|| {
                BotError::ServiceError(ServiceError::InstagramError(InstagramError::AuthenticationError(
                    AuthenticationError::LoginFailed("No user ID in response".into()),
                )))
            })?,
            username: credentials.username,
            authenticated: true,
            // Session cookies
            sessionid: self.extract_cookie("sessionid")?,
            ds_user_id: self.extract_cookie("ds_user_id")?,
            csrftoken: self.extract_cookie("csrftoken")?,
            // rur: self.extract_cookie("rur")?,
            ig_did: self.extract_cookie("ig_did")?,
            mid: self.extract_cookie("mid")?,
        };

        info!("Session data: {:?}", session_data);

        // Step 6: Verify session is valid
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        if !self.verify_session().await? {
            return Err(BotError::ServiceError(ServiceError::InstagramError(
                InstagramError::AuthenticationError(AuthenticationError::LoginFailed("Session invalid".into())),
            )));
        }

        Ok(session_data)
    }

    pub async fn verify_session(&self) -> BotResult<bool> {
        info!("Verifying session ...");
        if self.extract_session_id().is_err() {
            return Ok(false);
        }

        let response = self
            .client
            .get("https://www.instagram.com/accounts/edit/")
            .send()
            .await
            .map_err(|e| BotError::ServiceError(ServiceError::InstagramError(InstagramError::NetworkError(e))))?;

        Ok(response.status().is_success())
    }

    async fn perform_login(&self, username: &str, password: &str, csrf_token: &str) -> BotResult<LoginResponse> {
        info!("Performing login");
        let enc_password = format!("#PWD_INSTAGRAM_BROWSER:0:{}:{}", Utc::now().timestamp(), password);

        let form_data = [
            ("username", username),
            ("enc_password", &enc_password),
            ("queryParams", "{}"),
            ("optIntoOneTap", "false"),
            ("trustedDeviceRecords", "{}"),
        ];

        let response = self
            .client
            .post("https://www.instagram.com/api/v1/web/accounts/login/ajax/")
            .header("X-CSRFToken", csrf_token)
            .header("X-IG-App-ID", "936619743392459")
            .header("X-ASBD-ID", "198387")
            .header("X-IG-WWW-Claim", "0")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&form_data)
            .send()
            .await
            .map_err(|e| BotError::ServiceError(ServiceError::InstagramError(InstagramError::NetworkError(e))))?;

        let login_response = response.json::<LoginResponse>().await.map_err(|e| {
            BotError::ServiceError(ServiceError::InstagramError(InstagramError::DeserializationError(
                e.to_string(),
            )))
        })?;

        if login_response.status != "ok" {
            return Err(BotError::ServiceError(ServiceError::InstagramError(
                InstagramError::AuthenticationError(AuthenticationError::LoginFailed(
                    login_response.message.unwrap_or_else(|| "Authentication failed".into()),
                )),
            )));
        }

        Ok(login_response)
    }

    /// Get CSRF token from cookie jar
    async fn get_csrf_token(&self) -> BotResult<String> {
        info!("Getting CSRF token");
        if let Some(cookies) = self.cookie_jar.cookies(&"https://www.instagram.com".parse().unwrap()) {
            let cookie_str = cookies.to_str().unwrap();
            for cookie in cookie_str.split(';').map(|s| s.trim()) {
                let parts: Vec<&str> = cookie.split('=').collect();
                if parts.len() == 2 && parts[0] == "csrftoken" {
                    return Ok(parts[1].to_string());
                }
            }
        }
        Err(BotError::ServiceError(ServiceError::InstagramError(
            InstagramError::AuthenticationError(AuthenticationError::LoginFailed("Failed to get CSRF token".into())),
        )))
    }
    fn extract_cookie(&self, name: &str) -> BotResult<InstagramCookie> {
        let base_url = Url::parse("https://www.instagram.com").unwrap();

        if let Some(cookies) = self.cookie_jar.cookies(&base_url) {
            let cookie_str = cookies.to_str().unwrap();
            for cookie in cookie_str.split(';').map(|s| s.trim()) {
                let parts: Vec<&str> = cookie.split('=').collect();
                if parts.len() == 2 && parts[0] == name {
                    return Ok(InstagramCookie {
                        name: name.to_string(),
                        value: parts[1].to_string(),
                        domain: ".instagram.com".to_string(),
                        path: "/".to_string(),
                        expires: Some(Utc::now() + Duration::from_secs(365 * 24 * 60 * 60)),
                    });
                }
            }
        }

        Err(BotError::ServiceError(ServiceError::InstagramError(
            InstagramError::AuthenticationError(AuthenticationError::LoginFailed(format!(
                "Required cookie {} not found",
                name
            ))),
        )))
    }

    fn extract_session_id(&self) -> BotResult<InstagramCookie> {
        // get sessionid from cookie jar
        self.extract_cookie("sessionid")
    }

    fn create_client(cookie_store: Arc<Jar>) -> BotResult<reqwest::Client> {
        let builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(30))
            .cookie_provider(Arc::clone(&cookie_store))
            .default_headers(http::build_desktop_instagram_headers(false))
            .user_agent(http::INSTAGRAM_USER_AGENT);

        http::build_client(builder)
    }
}
