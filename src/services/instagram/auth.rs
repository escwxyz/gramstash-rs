use crate::{
    error::{AuthenticationError, BotError, BotResult, InstagramError, ServiceError},
    services::session::SessionData,
};

use super::InstagramService;
use chrono::Utc;
use reqwest::cookie::CookieStore;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub status: String,
    pub _authenticated: Option<bool>,
    pub _user: Option<bool>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    pub message: Option<String>,
    // pub two_factor_required: Option<bool>,
    // pub two_factor_info: Option<TwoFactorInfo>,
    // pub checkpoint_url: Option<String>,
}

// #[derive(Deserialize, Debug)]
// pub struct TwoFactorInfo {
//     pub two_factor_identifier: String,
// }

// #[derive(Clone)]
// pub struct TwoFactorAuthPending {
//     pub user: String,
//     pub two_factor_identifier: String,
// }

impl InstagramService {
    pub async fn login(&mut self, username: &str, password: &str) -> BotResult<SessionData> {
        // First visit the homepage to get initial cookies
        info!("Visiting homepage to get initial cookies");
        self.auth_client
            .get("https://www.instagram.com/")
            .send()
            .await
            .map_err(|e| BotError::ServiceError(ServiceError::InstagramError(InstagramError::NetworkError(e))))?;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Get CSRF token
        let csrf_token = self.get_csrf_token().await?;

        info!("Using CSRF token: {}", csrf_token);

        let login_response = self.perform_login(username, password, &csrf_token).await?;

        // Create session data
        let session_data = SessionData {
            cookies: self.extract_cookies(),
            user_id: login_response.user_id,
            username: Some(username.to_string()),
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

        info!("Login response status: {}", response.status());

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

    fn extract_cookie_value(&self, name: &str) -> Option<String> {
        self.cookie_jar
            .cookies(&"https://www.instagram.com".parse().unwrap())
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
        self.extract_cookie_value("sessionid")
    }

    pub async fn verify_session(&self) -> BotResult<bool> {
        if self.extract_session_id().is_none() {
            return Ok(false);
        }

        let response = self
            .auth_client
            .get("https://www.instagram.com/accounts/edit/")
            .send()
            .await
            .map_err(|e| BotError::ServiceError(ServiceError::InstagramError(InstagramError::NetworkError(e))))?;

        Ok(response.status().is_success())
    }
    // pub async fn two_factor_login(&mut self, code: &str) -> Result<()> {
    //     let pending = self
    //         .two_factor_auth_pending
    //         .as_ref()
    //         .ok_or_else(|| anyhow!("No two-factor authentication pending"))?;

    //     let two_factor_data = serde_json::json!({
    //         "username": pending.user,
    //         "verificationCode": code,
    //         "identifier": pending.two_factor_identifier,
    //         "csrf_token": self.session_data.csrf_token,
    //     });

    //     let response = self
    //         .client
    //         .post("https://www.instagram.com/accounts/login/ajax/two_factor/")
    //         .header("X-CSRFToken", self.session_data.csrf_token.as_deref().unwrap_or(""))
    //         .json(&two_factor_data)
    //         .send()
    //         .await?;

    //     let username = pending.user.clone();
    //     self.two_factor_auth_pending = None;

    //     let login_response = response.json::<LoginResponse>().await?;

    //     if login_response.authenticated.unwrap_or(false) {
    //         return Err(anyhow!("Two-factor authentication failed"));
    //     }
    //     // TODO
    //     // self.save_session(username, username, username).await?;
    //     Ok(())
    // }

    // pub async fn logout(&mut self) -> Result<(), BotError> {
    //     if let Some(username) = &self.session_data.username.clone() {
    //         info!("Logging out user: {}", username);

    //         // TODO
    //         // self.clear_session(username).await?;

    //         // Make logout request to Instagram
    //         let result = self
    //             .client
    //             .post("https://www.instagram.com/accounts/logout/")
    //             .send()
    //             .await;

    //         if let Err(e) = result {
    //             warn!("Failed to send logout request to Instagram: {}", e);
    //             // Continue with local cleanup even if Instagram request fails
    //         }
    //     } else {
    //         warn!("Attempted to logout with no active session");
    //     }

    //     Ok(())
    // }
}
