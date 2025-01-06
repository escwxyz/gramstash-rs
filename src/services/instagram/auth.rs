use std::sync::Arc;

use crate::{state::AppState, utils::error::BotError};

use super::{types::SerializableCookie, InstagramService, LoginResponse, SessionData, TwoFactorAuthPending};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use redis::AsyncCommands;
use reqwest::cookie::{CookieStore, Jar};

impl InstagramService {
    pub async fn login(&mut self, username: &str, password: &str) -> Result<()> {
        // First try to load existing session
        info!("Attempting to load existing session for {}", username);
        match self.load_session(username).await {
            Ok(true) => {
                info!("Existing session loaded successfully");
                // Verify the session is still valid
                if self.verify_session().await? {
                    info!("Session verified successfully");
                    return Ok(());
                }
                info!("Session invalid, proceeding with new login");
            }
            Ok(false) => {
                info!("No existing session found, proceeding with new login");
            }
            Err(e) => {
                warn!("Error loading session: {}, proceeding with new login", e);
            }
        }
        // First visit the homepage to get initial cookies
        info!("Visiting homepage to get initial cookies");
        self.client.get("https://www.instagram.com/").send().await?;

        // Wait a bit for cookies to be set
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        info!("Updating CSRF token");
        self.update_csrf_token().await?;

        let csrf_token = self
            .session_data
            .csrf_token
            .as_deref()
            .ok_or_else(|| anyhow!("No CSRF token available"))?;

        info!("Using CSRF token: {}", csrf_token);

        let enc_password = format!("#PWD_INSTAGRAM_BROWSER:0:{}:{}", Utc::now().timestamp(), password);

        let form_data = [
            ("username", username),
            ("enc_password", &enc_password),
            ("queryParams", "{}"),
            ("optIntoOneTap", "false"),
            ("trustedDeviceRecords", "{}"),
        ];

        info!("Sending login request");

        let response = self
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
            .context("Failed to send login request")?;

        info!("Response status: {}", response.status());
        info!("Response headers: {:?}", response.headers());

        let response_text = response.text().await.context("Failed to get response text")?;
        info!("Raw login response body: {}", response_text);

        let login_response: LoginResponse =
            serde_json::from_str(&response_text).context("Failed to parse login response")?;

        info!("Login response: {:?}", login_response);

        if login_response.status == "fail" {
            let error_message = login_response
                .message
                .unwrap_or_else(|| "Instagram rejected the login request".to_string());
            return Err(anyhow!(error_message));
        }

        self.handle_login_response(login_response, username).await?;
        self.save_session(username).await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn two_factor_login(&mut self, code: &str) -> Result<()> {
        let pending = self
            .two_factor_auth_pending
            .as_ref()
            .ok_or_else(|| anyhow!("No two-factor authentication pending"))?;

        let two_factor_data = serde_json::json!({
            "username": pending.user,
            "verificationCode": code,
            "identifier": pending.two_factor_identifier,
            "csrf_token": self.session_data.csrf_token,
        });

        let response = self
            .client
            .post("https://www.instagram.com/accounts/login/ajax/two_factor/")
            .header("X-CSRFToken", self.session_data.csrf_token.as_deref().unwrap_or(""))
            .json(&two_factor_data)
            .send()
            .await?;

        let username = pending.user.clone();
        self.two_factor_auth_pending = None;

        let login_response = response.json::<LoginResponse>().await?;

        if login_response.authenticated.unwrap_or(false) {
            return Err(anyhow!("Two-factor authentication failed"));
        }

        self.save_session(&username).await?;
        Ok(())
    }

    pub async fn logout(&mut self) -> Result<(), BotError> {
        if let Some(username) = &self.session_data.username.clone() {
            info!("Logging out user: {}", username);

            // Clear Redis session
            let mut conn = AppState::get()
                .redis
                .get_connection()
                .await
                .map_err(|e| BotError::RedisError(format!("Failed to connect to Redis: {}", e)))?;

            let key = format!("instagram_session:{}", username);
            conn.del::<_, ()>(&key)
                .await
                .map_err(|e| BotError::CacheError(format!("Failed to delete session from Redis: {}", e)))?;

            info!("Cleared Redis session for user: {}", username);

            // Make logout request to Instagram
            let result = self
                .client
                .post("https://www.instagram.com/accounts/logout/")
                .send()
                .await;

            if let Err(e) = result {
                warn!("Failed to send logout request to Instagram: {}", e);
                // Continue with local cleanup even if Instagram request fails
            }

            // Clear local session data
            self.session_data = SessionData::default();
            self.cookie_jar = Arc::new(Jar::default());

            info!("Cleared local session data for user: {}", username);
        } else {
            warn!("Attempted to logout with no active session");
        }

        Ok(())
    }

    pub fn get_username(&self) -> Option<String> {
        self.session_data.username.clone()
    }

    async fn load_session(&mut self, username: &str) -> Result<bool, BotError> {
        let state = AppState::get();
        let mut conn = state
            .redis
            .get_connection()
            .await
            .map_err(|e| BotError::RedisError(e.to_string()))?;

        let key = format!("instagram_session:{}", username);

        let session_data: Option<String> = conn.get(&key).await.map_err(|e| BotError::CacheError(e.to_string()))?;

        if let Some(data) = session_data {
            let session: SessionData = serde_json::from_str(&data)
                .map_err(|e| BotError::InstagramApi(format!("Failed to parse session data: {}", e)))?;

            // Restore cookies to cookie jar
            for cookie in &session.cookies {
                let cookie_string = format!(
                    "{}={}; Domain={}; Path={}",
                    cookie.name, cookie.value, cookie.domain, cookie.path
                );
                self.cookie_jar
                    .add_cookie_str(&cookie_string, &"https://www.instagram.com".parse().unwrap());
            }

            self.session_data = session;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn verify_session(&self) -> Result<bool, BotError> {
        // Make a simple request that requires authentication
        let response = self
            .client
            .get("https://www.instagram.com/accounts/edit/")
            .send()
            .await
            .map_err(|e| BotError::InstagramApi(format!("Failed to verify session: {}", e)))?;

        Ok(response.status().is_success())
    }

    async fn save_session(&self, username: &str) -> Result<(), BotError> {
        let state = AppState::get();
        let mut conn = state
            .redis
            .get_connection()
            .await
            .map_err(|e| BotError::RedisError(e.to_string()))?;
        let key = format!("instagram_session:{}", username);

        // Get current cookies
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

        // Try to get existing session
        let existing_session: Option<String> = conn.get(&key).await.map_err(|e| BotError::CacheError(e.to_string()))?;

        let session_data = if let Some(existing) = existing_session {
            // Update existing session with new data
            let mut existing_session: SessionData = serde_json::from_str(&existing)
                .map_err(|e| BotError::CacheError(format!("Failed to parse existing session: {}", e)))?;

            // Update only non-None values
            existing_session.cookies = cookies;
            if self.session_data.user_id.is_some() {
                existing_session.user_id = self.session_data.user_id.clone();
            }
            if self.session_data.username.is_some() {
                existing_session.username = self.session_data.username.clone();
            }
            if self.session_data.csrf_token.is_some() {
                existing_session.csrf_token = self.session_data.csrf_token.clone();
            }
            if self.session_data.session_id.is_some() {
                existing_session.session_id = self.session_data.session_id.clone();
            }
            if self.session_data.device_id.is_some() {
                existing_session.device_id = self.session_data.device_id.clone();
            }
            if self.session_data.machine_id.is_some() {
                existing_session.machine_id = self.session_data.machine_id.clone();
            }
            if self.session_data.rur.is_some() {
                existing_session.rur = self.session_data.rur.clone();
            }
            existing_session
        } else {
            // Create new session
            SessionData {
                cookies,
                user_id: self.session_data.user_id.clone(),
                username: self.session_data.username.clone(),
                csrf_token: self.session_data.csrf_token.clone(),
                session_id: self.session_data.session_id.clone(),
                device_id: self.session_data.device_id.clone(),
                machine_id: self.session_data.machine_id.clone(),
                rur: self.session_data.rur.clone(),
            }
        };

        let serialized = serde_json::to_string(&session_data)
            .map_err(|e| BotError::CacheError(format!("Failed to serialize session: {}", e)))?;

        conn.set::<_, _, String>(&key, serialized)
            .await
            .map_err(|e| BotError::RedisError(format!("Failed to set session data in Redis: {}", e)))?;

        Ok(())
    }

    async fn handle_login_response(&mut self, login_response: LoginResponse, username: &str) -> Result<()> {
        // Handle two-factor auth first
        if login_response.two_factor_required.unwrap_or(false) {
            if let Some(two_factor_info) = login_response.two_factor_info {
                self.two_factor_auth_pending = Some(TwoFactorAuthPending {
                    user: username.to_string(),
                    two_factor_identifier: two_factor_info.two_factor_identifier,
                });
                return Err(anyhow!("Two factor authentication required"));
            }
        }

        if let Some(checkpoint_url) = login_response.checkpoint_url {
            return Err(anyhow!("Checkpoint required: {}", checkpoint_url));
        }

        if !login_response.authenticated.unwrap_or(false) {
            if login_response.user.unwrap_or(false) {
                return Err(anyhow!("Bad credentials"));
            } else {
                return Err(anyhow!("User {} does not exist", username));
            }
        }

        let cookie_store = self.cookie_jar.cookies(&"https://www.instagram.com".parse().unwrap());
        if let Some(cookies) = cookie_store {
            let cookie_str = cookies.to_str().unwrap();
            for cookie in cookie_str.split(';').map(|s| s.trim()) {
                let parts: Vec<&str> = cookie.split('=').collect();
                if parts.len() == 2 {
                    match parts[0] {
                        "sessionid" => self.session_data.session_id = Some(parts[1].to_string()),
                        "csrftoken" => self.session_data.csrf_token = Some(parts[1].to_string()),
                        "ig_did" => self.session_data.device_id = Some(parts[1].to_string()),
                        "mid" => self.session_data.machine_id = Some(parts[1].to_string()),
                        "rur" => self.session_data.rur = Some(parts[1].to_string()),
                        _ => {}
                    }
                }
            }
        }

        self.session_data.username = Some(username.to_string());
        self.session_data.user_id = login_response.user_id;

        info!(
            "Login successful for user: {}, user_id: {:?}",
            username, self.session_data.user_id
        );

        Ok(())
    }

    async fn update_csrf_token(&mut self) -> Result<()> {
        let response = self.client.get("https://www.instagram.com/").send().await?;

        // Log all cookies for debugging
        info!("All cookies from response:");
        for cookie in response.cookies() {
            info!("Cookie: {}={}", cookie.name(), cookie.value());
            if cookie.name() == "csrftoken" {
                self.session_data.csrf_token = Some(cookie.value().to_string());
                info!("Found CSRF token: {}", cookie.value());
            }
        }

        // Double check if we got the token
        if self.session_data.csrf_token.is_none() {
            // Try to get it from cookie jar
            if let Some(cookies) = self.cookie_jar.cookies(&"https://www.instagram.com".parse().unwrap()) {
                let cookie_str = cookies.to_str().unwrap();
                info!("Cookie jar contents: {}", cookie_str);
                for cookie in cookie_str.split(';').map(|s| s.trim()) {
                    let parts: Vec<&str> = cookie.split('=').collect();
                    if parts.len() == 2 && parts[0] == "csrftoken" {
                        self.session_data.csrf_token = Some(parts[1].to_string());
                        info!("Found CSRF token from cookie jar: {}", parts[1]);
                        break;
                    }
                }
            }
        }

        if self.session_data.csrf_token.is_none() {
            return Err(anyhow!("Failed to obtain CSRF token"));
        }

        Ok(())
    }
}
