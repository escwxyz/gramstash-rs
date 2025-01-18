use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};

use crate::{
    error::{BotError, BotResult},
    state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    English,
    Chinese,
    German,
    French,
    Japanese,
    Spanish,
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Ok(Language::English),
            "zh" | "chinese" => Ok(Language::Chinese),
            "de" | "german" => Ok(Language::German),
            "fr" | "french" => Ok(Language::French),
            "ja" | "japanese" => Ok(Language::Japanese),
            "es" | "spanish" => Ok(Language::Spanish),
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
            Language::French => "fr".to_string(),
            Language::Japanese => "ja".to_string(),
            Language::Spanish => "es".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LanguageService {
    pub language_cache: Arc<DashMap<String, Language>>,
}

impl LanguageService {
    pub fn new(capacity: usize) -> BotResult<Self> {
        info!("Initializing LanguageService...");
        let language_cache = Arc::new(DashMap::with_capacity(capacity));
        info!("LanguageService initialized");
        Ok(Self { language_cache })
    }

    pub async fn get_user_language(&self, telegram_user_id: &str) -> BotResult<Language> {
        if let Some(lang) = self.language_cache.get(telegram_user_id) {
            return Ok(*lang);
        }

        let lang = self.load_language_from_database(telegram_user_id).await?;

        self.language_cache.insert(telegram_user_id.to_string(), lang);

        Ok(lang)
    }

    async fn load_language_from_database(&self, telegram_user_id: &str) -> BotResult<Language> {
        let app_state = AppState::get()?;
        let conn = app_state.turso.get_connection().await?;
        let mut rows = conn
            .query(
                "SELECT language FROM user_language WHERE telegram_user_id = ?1 LIMIT 1",
                [telegram_user_id],
            )
            .await
            .map_err(|e| BotError::TursoError(e.to_string()))?;

        while let Some(row) = rows.next().await.map_err(|e| BotError::TursoError(e.to_string()))? {
            let language = row.get::<String>(0).unwrap_or_else(|_| "en".to_string());
            return Ok(Language::from_str(&language).unwrap_or(Language::English));
        }

        Ok(Language::English)
    }

    pub async fn set_user_language(&self, telegram_user_id: &str, language: Language) -> BotResult<()> {
        self.language_cache.insert(telegram_user_id.to_string(), language);

        Ok(())
    }

    pub async fn save_languages_to_database(&self) -> BotResult<()> {
        let app_state = AppState::get()?;
        let conn = app_state.turso.get_connection().await?;

        // Start transaction
        let tx = conn
            .transaction()
            .await
            .map_err(|e| BotError::TursoError(e.to_string()))?;

        // Prepare batch values
        let mut values = Vec::new();
        let mut params = Vec::new();

        for entry in self.language_cache.iter() {
            let telegram_user_id = entry.key();
            let language = entry.value().clone();
            values.push(format!("(?{}, ?{})", params.len() + 1, params.len() + 2));
            params.push(telegram_user_id.clone());
            params.push(language.to_string());
        }

        if !values.is_empty() {
            // Build and execute batch query
            let query = format!(
                "INSERT OR REPLACE INTO user_language (telegram_user_id, language) VALUES {}",
                values.join(",")
            );

            tx.execute(&query, params)
                .await
                .map_err(|e| BotError::TursoError(e.to_string()))?;
        }

        // Commit transaction
        tx.commit().await.map_err(|e| BotError::TursoError(e.to_string()))?;

        Ok(())
    }
}
