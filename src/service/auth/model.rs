use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::platform::Platform;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthData {
    pub cookies: HashMap<String, CookieData>,
    pub tokens: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)] // TODO: use serde_json::Value, to implement TryFrom trait
pub struct CookieData {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Credentials {
    pub indentifier: String,
    pub password: String,
    pub platform: Platform,
    #[allow(dead_code)]
    pub two_factor_token: Option<String>,
}
