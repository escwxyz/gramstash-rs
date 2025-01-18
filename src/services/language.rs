use dashmap::DashMap;
use libsql::params;
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
    pub interface_cache: Arc<DashMap<String, String>>,
}

impl LanguageService {
    pub fn new(capacity: usize) -> BotResult<Self> {
        info!("Initializing LanguageService...");
        let language_cache = Arc::new(DashMap::with_capacity(capacity));
        let interface_cache = Arc::new(DashMap::with_capacity(capacity));
        info!("LanguageService initialized");
        Ok(Self {
            language_cache,
            interface_cache,
        })
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
        self.save_language_to_database(telegram_user_id, language).await?;

        self.language_cache.insert(telegram_user_id.to_string(), language);

        Ok(())
    }

    async fn save_language_to_database(&self, telegram_user_id: &str, language: Language) -> BotResult<()> {
        let app_state = AppState::get()?;
        let conn = app_state.turso.get_connection().await?;
        conn.execute(
            "INSERT OR REPLACE INTO user_language (telegram_user_id, language) VALUES (?1, ?2)",
            params![telegram_user_id, language.to_string()],
        )
        .await
        .map_err(|e| BotError::TursoError(e.to_string()))?;

        Ok(())
    }

    pub async fn set_last_interface(&self, user_id: &str, interface: &str) -> BotResult<()> {
        self.save_interface_to_database(user_id, interface).await?;

        self.interface_cache.insert(user_id.to_string(), interface.to_string());

        Ok(())
    }

    async fn save_interface_to_database(&self, user_id: &str, interface: &str) -> BotResult<()> {
        let app_state = AppState::get()?;
        let conn = app_state.turso.get_connection().await?;
        conn.execute(
            "INSERT OR REPLACE INTO user_last_interface (telegram_user_id, interface) VALUES (?1, ?2)",
            params![user_id, interface],
        )
        .await
        .map_err(|e| BotError::TursoError(e.to_string()))?;
        Ok(())
    }

    pub async fn get_last_interface(&self, user_id: &str) -> BotResult<String> {
        if let Some(interface) = self.interface_cache.get(user_id) {
            return Ok(interface.clone());
        }

        let interface = self.load_interface_from_database(user_id).await?;
        self.interface_cache.insert(user_id.to_string(), interface.clone());
        Ok(interface)
    }

    async fn load_interface_from_database(&self, user_id: &str) -> BotResult<String> {
        let app_state = AppState::get()?;
        let conn = app_state.turso.get_connection().await?;
        let mut rows = conn
            .query(
                "SELECT interface FROM user_last_interface WHERE telegram_user_id = ?1 LIMIT 1",
                [user_id],
            )
            .await
            .map_err(|e| BotError::TursoError(e.to_string()))?;

        while let Some(row) = rows.next().await.map_err(|e| BotError::TursoError(e.to_string()))? {
            let interface = row.get::<String>(0).unwrap_or_else(|_| "main".to_string());
            return Ok(interface);
        }

        Ok("main".to_string())
    }
}

// TODO: Add tests
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_language_operations() {
//         let service = LanguageService::new();
//         let user_id = "test_user_123";

//         // Test setting language
//         service
//             .set_user_language(user_id.to_string(), Language::German)
//             .await
//             .expect("Failed to set language");

//         // Test getting language
//         let language = service
//             .get_user_language(user_id)
//             .await
//             .expect("Failed to get language");
//         assert_eq!(language, Language::German);
//     }

//     #[tokio::test]
//     async fn test_interface_operations() {
//         let service = LanguageService::new();
//         let user_id = "test_user_123";
//         let interface = "test_interface";

//         // Test setting interface
//         service
//             .set_last_interface(user_id, interface)
//             .await
//             .expect("Failed to set interface");

//         // Test getting interface
//         let result = service
//             .get_last_interface(user_id)
//             .await
//             .expect("Failed to get interface");
//         assert_eq!(result, interface);
//     }
// }
