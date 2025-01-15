use dashmap::DashMap;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::{error::BotResult, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    English,
    #[allow(dead_code)]
    Chinese,
    #[allow(dead_code)]
    German,
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Ok(Language::English),
            "zh" | "chinese" => Ok(Language::Chinese),
            "de" | "german" => Ok(Language::German),
            _ => Err(format!("Unknown language code: {}", s)),
        }
    }
}

impl ToString for Language {
    fn to_string(&self) -> String {
        match self {
            Language::English => "en".to_string(),
            Language::Chinese => "zh".to_string(),
            Language::German => "de".to_string(),
        }
    }
}

impl Language {
    pub async fn set_user_language(app_state: &AppState, telegram_user_id: &str, language: Language) -> BotResult<()> {
        app_state
            .language
            .set_user_language(telegram_user_id.to_string(), language);

        Ok(())
    }

    pub async fn get_user_language(app_state: &AppState, telegram_user_id: &str) -> BotResult<Language> {
        Ok(app_state.language.get_user_language(telegram_user_id))
    }
}

#[derive(Debug, Clone)]
pub struct LanguageService {
    cache: DashMap<String, Language>,
    last_interface: DashMap<String, String>,
}
impl LanguageService {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
            last_interface: DashMap::new(),
        }
    }

    pub fn get_user_language(&self, telegram_user_id: &str) -> Language {
        self.cache
            .get(telegram_user_id)
            .map(|lang| *lang)
            .unwrap_or(Language::English)
    }

    pub fn set_user_language(&self, telegram_user_id: String, language: Language) {
        self.cache.insert(telegram_user_id, language);
    }

    pub fn set_last_interface(&self, user_id: &str, interface: &str) {
        self.last_interface.insert(user_id.to_string(), interface.to_string());
    }

    pub fn get_last_interface(&self, user_id: &str) -> String {
        self.last_interface
            .get(user_id)
            .map(|v| v.clone())
            .unwrap_or_else(|| "main".to_string())
    }

    #[allow(unused)]
    /// Background task to persist user language to redis for metrics
    pub async fn persist_user_language(&self, telegram_user_id: String, language: Language) -> BotResult<()> {
        let app_state = AppState::get()?;
        let mut conn = app_state.redis.get_connection().await?;
        conn.set::<_, _, String>(format!("user_language:{}", telegram_user_id), language.to_string())
            .await?;

        Ok(())
    }
}
