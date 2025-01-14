use chrono::Utc;
use reqwest::cookie::CookieStore;
use url::Url;

use crate::error::{AuthenticationError, BotError, BotResult, InstagramError, ServiceError};

use super::{
    client::AuthClient,
    session::SessionService,
    types::{Credentials, LoginResponse, SessionData},
};

#[derive(Clone)]
pub struct AuthService {
    pub auth_client: AuthClient,
    pub session_service: SessionService,
}

impl AuthService {
    pub fn new(session_service: SessionService) -> BotResult<Self> {
        let auth_client = AuthClient::new()?;
        Ok(Self {
            auth_client,
            session_service,
        })
    }

    pub async fn login(&mut self, credentials: Credentials) -> BotResult<SessionData> {
        info!("Visiting homepage to get initial cookies");
        self.auth_client
            .client
            .get("https://www.instagram.com/")
            .send()
            .await
            .map_err(|e| BotError::ServiceError(ServiceError::InstagramError(InstagramError::NetworkError(e))))?;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Get CSRF token
        let csrf_token = self.get_csrf_token().await?;

        info!("Using CSRF token: {}", csrf_token);

        let login_response = self
            .perform_login(&credentials.username, &credentials.password, &csrf_token)
            .await?;

        // Create session data
        let session_data = SessionData {
            cookies: self.auth_client.extract_cookies(),
            user_id: login_response.user_id,
            username: Some(credentials.username.to_string()),
            csrf_token: Some(csrf_token),
            session_id: self.extract_session_id(),
            device_id: self.extract_cookie_value("ig_did"),
            machine_id: self.extract_cookie_value("mid"),
            rur: self.extract_cookie_value("rur"),
        };

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        self.verify_session().await?;

        Ok(session_data)
    }

    pub async fn is_authenticated(&self, telegram_user_id: &str) -> BotResult<bool> {
        if self.session_service.session.belongs_to(telegram_user_id)
            && !self.session_service.needs_refresh()
            && self.session_service.session.session_data.is_some()
        {
            return Ok(true);
        }

        // If not, validate against stored session
        // TODO: implement this
        // self.session_service.validate_session(telegram_user_id).await
        Ok(false)
    }

    async fn perform_login(&mut self, username: &str, password: &str, csrf_token: &str) -> BotResult<LoginResponse> {
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
            .auth_client
            .client
            .post("https://www.instagram.com/api/v1/web/accounts/login/ajax/")
            .header("X-CSRFToken", csrf_token)
            .header("X-Requested-With", "XMLHttpRequest")
            .header("X-IG-WWW-Claim", "0")
            .header("Sec-Fetch-Site", "same-origin")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Dest", "empty")
            .form(&form_data)
            .send()
            .await
            .map_err(|e| BotError::ServiceError(ServiceError::InstagramError(InstagramError::NetworkError(e))))?;

        let login_response = response.json::<LoginResponse>().await.map_err(|e| {
            BotError::ServiceError(ServiceError::InstagramError(InstagramError::DeserializationError(
                e.to_string(),
            )))
        })?;

        info!("Login response: {:?}", login_response);

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
    pub(super) async fn get_csrf_token(&self) -> BotResult<String> {
        info!("Getting CSRF token");
        if let Some(cookies) = self
            .auth_client
            .cookie_jar
            .cookies(&"https://www.instagram.com".parse().unwrap())
        {
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

    fn extract_cookie_value(&self, name: &str) -> Option<String> {
        self.auth_client
            .cookie_jar
            .cookies(&Url::parse("https://www.instagram.com").unwrap())
            .and_then(|cookies| {
                cookies.to_str().ok().and_then(|cookie_str| {
                    cookie_str
                        .split(';')
                        .find(|c| c.trim().starts_with(name))
                        .and_then(|c| c.split('=').nth(1))
                        .map(String::from)
                })
            })
    }

    fn extract_session_id(&self) -> Option<String> {
        // get sessionid from cookie jar
        self.extract_cookie_value("sessionid")
    }

    pub async fn verify_session(&self) -> BotResult<bool> {
        if self.extract_session_id().is_none() {
            return Ok(false);
        }

        let response = self
            .auth_client
            .client
            .get("https://www.instagram.com/accounts/edit/")
            .send()
            .await
            .map_err(|e| BotError::ServiceError(ServiceError::InstagramError(InstagramError::NetworkError(e))))?;

        Ok(response.status().is_success())
    }
}
