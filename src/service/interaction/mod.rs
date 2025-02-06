use chrono::{Duration, Utc};

use crate::{
    error::BotResult,
    runtime::{CacheManager, CacheOptions, CacheType},
    state::AppState,
    storage::StorageError,
};

mod model;
pub use model::LastInterfaceState;

#[derive(Clone)]
pub struct InteractionService {
    cache: CacheManager,
    interface_lifespan: Duration,
}

impl InteractionService {
    pub async fn new(capacity: usize, interface_lifespan_secs: i64) -> BotResult<Self> {
        info!("Initializing InteractionService...");
        let cache = CacheManager::new(capacity)?;

        info!("InteractionService initialized");
        Ok(Self {
            cache,
            interface_lifespan: Duration::seconds(interface_lifespan_secs),
        })
    }

    pub async fn set_last_interface(&self, telegram_user_id: &str, interface: &str) -> BotResult<()> {
        let cache_options = CacheOptions {
            cache_type: CacheType::Memory,
            ttl: None,
            prefix: Some("interface".to_string()),
        };

        let state = LastInterfaceState {
            last_access: Utc::now(),
            interface: interface.to_string(),
        };

        self.cache
            .set::<LastInterfaceState>(telegram_user_id, state, &cache_options)
            .await?;

        Ok(())
    }

    pub async fn get_last_interface(&self, telegram_user_id: &str) -> BotResult<Option<LastInterfaceState>> {
        let cache_options = CacheOptions {
            cache_type: CacheType::Memory,
            ttl: None,
            prefix: Some("interface".to_string()),
        };

        let result = self.cache.get(telegram_user_id, &cache_options).await?;

        Ok(result)
    }

    #[allow(dead_code)]
    pub async fn cleanup_old_entries(&self) {
        let now = Utc::now();
        let cache_options = CacheOptions {
            cache_type: CacheType::Memory,
            ttl: None,
            prefix: Some("interface".to_string()),
        };

        if let Ok(keys) = self.cache.keys("*", &cache_options).await {
            for key in keys {
                if let Ok(Some(state)) = self.cache.get::<LastInterfaceState>(&key, &cache_options).await {
                    if now - state.last_access > self.interface_lifespan {
                        let _ = self.cache.del(&key, &cache_options).await;
                    }
                }
            }
        }
    }
    #[allow(dead_code)]
    pub async fn save_interfaces_to_database(&self) -> BotResult<()> {
        let app_state = AppState::get()?;
        let conn = app_state.storage.turso().get_connection().await?;

        let cache_options = CacheOptions {
            cache_type: CacheType::Memory,
            ttl: None,
            prefix: Some("interface".to_string()),
        };

        let keys = self.cache.keys("*", &cache_options).await?;
        if keys.is_empty() {
            return Ok(());
        }

        let tx = conn.transaction().await.map_err(|e| StorageError::Turso(e))?;

        let mut values = Vec::new();
        let mut params = Vec::new();

        for key in keys {
            if let Some(state) = self.cache.get::<LastInterfaceState>(&key, &cache_options).await? {
                let interface_str = format!("{}:{:?}", state.interface, state.last_access);
                values.push(format!("(?{}, ?{})", params.len() + 1, params.len() + 2));
                params.push(key);
                params.push(interface_str);
            }
        }

        if !values.is_empty() {
            let query = format!(
                "INSERT OR REPLACE INTO user_last_interface (telegram_user_id, interface) VALUES {}",
                values.join(",")
            );

            tx.execute(&query, params).await.map_err(|e| StorageError::Turso(e))?;
        }

        tx.commit().await.map_err(|e| StorageError::Turso(e))?;

        Ok(())
    }
}
