use std::str::FromStr;

pub use model::Language;

use crate::{
    error::BotResult,
    runtime::{CacheManager, CacheOptions, CacheType},
    state::AppState,
    storage::StorageError,
};

mod model;

#[derive(Clone)]
pub struct LanguageService {
    cache: CacheManager,
}

impl LanguageService {
    pub async fn new(capacity: usize) -> Result<Self, StorageError> {
        info!("Initializing LanguageService...");
        let cache = CacheManager::new(capacity)?;
        info!("LanguageService initialized");
        Ok(Self { cache })
    }
    #[allow(dead_code)]
    pub async fn get_user_language(&self, telegram_user_id: &str) -> BotResult<Language> {
        let cache_options = CacheOptions {
            cache_type: CacheType::Memory,
            ttl: None,
            prefix: Some("lang".to_string()),
        };

        if let Some(lang) = self.cache.get(telegram_user_id, &cache_options).await? {
            return Ok(lang);
        }

        let lang = self.load_language_from_database(telegram_user_id).await?;
        self.cache
            .set::<Language>(telegram_user_id, lang, &cache_options)
            .await?;

        Ok(lang)
    }

    async fn load_language_from_database(&self, telegram_user_id: &str) -> BotResult<Language> {
        let app_state = AppState::get()?;
        let conn = app_state.storage.turso().get_connection().await?;
        let mut rows = conn
            .query(
                "SELECT language FROM user_language WHERE telegram_user_id = ?1 LIMIT 1",
                [telegram_user_id],
            )
            .await
            .map_err(|e| StorageError::Turso(e))?;

        while let Some(row) = rows.next().await.map_err(|e| StorageError::Turso(e))? {
            let language = row.get::<String>(0).unwrap_or_else(|_| "en".to_string());
            return Ok(Language::from_str(&language).unwrap_or(Language::English));
        }

        Ok(Language::English)
    }

    pub async fn set_user_language(&self, telegram_user_id: &str, language: Language) -> BotResult<()> {
        let cache_options = CacheOptions {
            cache_type: CacheType::Memory,
            ttl: None,
            prefix: Some("lang".to_string()),
        };

        self.cache
            .set::<Language>(telegram_user_id, language, &cache_options)
            .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn save_languages_to_database(&self) -> BotResult<()> {
        let app_state = AppState::get()?;
        let conn = app_state.storage.turso().get_connection().await?;

        let cache_options = CacheOptions {
            cache_type: CacheType::Memory,
            ttl: None,
            prefix: Some("lang".to_string()),
        };

        let keys = self.cache.keys("*", &cache_options).await?;
        if keys.is_empty() {
            return Ok(());
        }

        let tx = conn.transaction().await.map_err(|e| StorageError::Turso(e))?;

        let mut values = Vec::new();
        let mut params = Vec::new();

        for key in keys {
            if let Some(language) = self.cache.get::<Language>(&key, &cache_options).await? {
                values.push(format!("(?{}, ?{})", params.len() + 1, params.len() + 2));
                params.push(key);
                params.push(language.to_string());
            }
        }

        if !values.is_empty() {
            let query = format!(
                "INSERT OR REPLACE INTO user_language (telegram_user_id, language) VALUES {}",
                values.join(",")
            );

            tx.execute(&query, params).await.map_err(|e| StorageError::Turso(e))?;
        }

        tx.commit().await.map_err(|e| StorageError::Turso(e))?;

        Ok(())
    }
}
