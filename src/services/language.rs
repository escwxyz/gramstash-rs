use libsql::params;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::{
    error::{BotError, BotResult},
    state::AppState,
};

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

#[derive(Debug, Clone)]
pub struct LanguageService;

impl LanguageService {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_user_language(&self, telegram_user_id: &str) -> BotResult<Language> {
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
        // self.cache
        //     .get(telegram_user_id)
        //     .map(|lang| *lang)
        //     .unwrap_or(Language::English)

        Ok(Language::English)
    }

    pub async fn set_user_language(&self, telegram_user_id: String, language: Language) -> BotResult<()> {
        let app_state = AppState::get()?;
        let conn = app_state.turso.get_connection().await?;
        conn.execute(
            "INSERT OR REPLACE INTO user_language (telegram_user_id, language) VALUES (?1, ?2)",
            params![telegram_user_id, language.to_string()],
        )
        .await
        .map_err(|e| BotError::TursoError(e.to_string()))?;
        // self.cache.insert(telegram_user_id, language);
        Ok(())
    }

    pub async fn set_last_interface(&self, user_id: &str, interface: &str) -> BotResult<()> {
        let app_state = AppState::get()?;
        let conn = app_state.turso.get_connection().await?;
        conn.execute(
            "INSERT OR REPLACE INTO user_last_interface (telegram_user_id, interface) VALUES (?1, ?2)",
            params![user_id, interface],
        )
        .await
        .map_err(|e| BotError::TursoError(e.to_string()))?;
        // self.last_interface.insert(user_id.to_string(), interface.to_string());
        Ok(())
    }

    pub async fn get_last_interface(&self, user_id: &str) -> BotResult<String> {
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

        // self.last_interface
        //     .get(user_id)
        //     .map(|v| v.clone())
        //     .unwrap_or_else(|| "main".to_string())
        Ok("main".to_string())
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
