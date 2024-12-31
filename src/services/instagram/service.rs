use std::sync::Arc;

use reqwest::{cookie::Jar, Client};

use crate::utils::http;

use super::{SessionData, TwoFactorAuthPending};

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
        };

        Self {
            client,
            cookie_jar,
            session_data,
            two_factor_auth_pending: None,
        }
    }
}
