use std::sync::Arc;

use crate::state::AppState;

use super::{types::SerializableCookie, InstagramService, LoginResponse, SessionData, TwoFactorAuthPending};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use redis::AsyncCommands;
use reqwest::cookie::{CookieStore, Jar};

impl InstagramService {
    pub async fn login(&mut self, username: &str, password: &str) -> Result<()> {
        info!("Loading session for {}", username);
        if self.load_session(username).await? {
            return Ok(());
        }

        info!("Updating CSRF token");
        self.update_csrf_token().await?;

        let enc_password = format!("#PWD_INSTAGRAM_BROWSER:0:{}:{}", Utc::now().timestamp(), password);

        let login_data = serde_json::json!({
            "enc_password": enc_password,
            "username": username,
            "csrf_token": self.session_data.csrf_token,
        });

        info!("Sending login request with data: {:?}", login_data);

        let response = self
            .client
            .post("https://www.instagram.com/api/v1/web/accounts/login/ajax/")
            .header("X-CSRFToken", self.session_data.csrf_token.as_deref().unwrap_or(""))
            .json(&login_data)
            .send()
            .await
            .context("Failed to send login request")?;

        self.handle_login_response(response, username).await?;
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

        if !login_response.authenticated {
            return Err(anyhow!("Two-factor authentication failed"));
        }

        self.save_session(&username).await?;
        Ok(())
    }

    pub async fn logout(&mut self) -> Result<()> {
        if let Some(username) = &self.session_data.username {
            // Clear Redis session
            let mut conn = AppState::get().redis.get_connection().await?;
            let key = format!("instagram_session:{}", username);
            conn.del::<_, ()>(key).await?;

            // Clear local session data
            self.session_data = SessionData {
                cookies: Vec::new(),
                user_id: None,
                username: None,
                csrf_token: None,
            };

            // Clear cookie jar
            self.cookie_jar = Arc::new(Jar::default());
        }
        Ok(())
    }

    pub fn is_logged_in(&self) -> bool {
        self.session_data.username.is_some()
    }

    async fn load_session(&mut self, username: &str) -> Result<bool> {
        let state = AppState::get();
        let mut conn = state.redis.get_connection().await?;
        let key = format!("instagram_session:{}", username);

        let session_data: Option<String> = conn.get(&key).await?;

        if let Some(data) = session_data {
            let session: SessionData =
                serde_json::from_str(&data).map_err(|e| anyhow!("Failed to parse session data: {}", e))?;

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

    async fn save_session(&self, username: &str) -> Result<()> {
        let state = AppState::get();
        let mut conn = state.redis.get_connection().await?;
        let key = format!("instagram_session:{}", username);

        // Extract cookies from cookie jar
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

        let session_data = SessionData {
            cookies,
            user_id: self.session_data.user_id.clone(),
            username: self.session_data.username.clone(),
            csrf_token: self.session_data.csrf_token.clone(),
        };

        let serialized =
            serde_json::to_string(&session_data).map_err(|e| anyhow!("Failed to serialize session: {}", e))?;

        // Store in Redis with 30-day expiration
        conn.set_ex::<_, _, String>(&key, serialized, 30 * 24 * 60 * 60)
            .await
            .context("Failed to set session data in Redis")?;

        Ok(())
    }

    async fn handle_login_response(&mut self, response: reqwest::Response, username: &str) -> Result<()> {
        let login_response = response
            .json::<LoginResponse>()
            .await
            .map_err(|_| anyhow!("Failed to parse response"))?;

        if login_response.two_factor_required.unwrap_or(false) {
            if let Some(two_factor_info) = login_response.two_factor_info {
                self.two_factor_auth_pending = Some(TwoFactorAuthPending {
                    user: username.to_string(),
                    two_factor_identifier: two_factor_info.two_factor_identifier,
                });
                return Err(anyhow!("Two factor required"));
            }
        }

        if let Some(checkpoint_url) = login_response.checkpoint_url {
            return Err(anyhow!("Checkpoint required: {}", checkpoint_url));
        }

        if !login_response.authenticated {
            if login_response.user {
                return Err(anyhow!("Bad credentials"));
            } else {
                return Err(anyhow!("User {} does not exist", username));
            }
        }

        // Store user information
        self.session_data.username = Some(username.to_string());
        self.session_data.user_id = login_response.user_id;

        Ok(())
    }

    async fn update_csrf_token(&mut self) -> Result<()> {
        let response = self.client.get("https://www.instagram.com/").send().await?;

        let cookies = response.cookies();
        for cookie in cookies {
            if cookie.name() == "csrftoken" {
                self.session_data.csrf_token = Some(cookie.value().to_string());
                break;
            }
        }

        Ok(())
    }
}
