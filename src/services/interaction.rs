use chrono::{DateTime, Duration, TimeDelta, Utc};
use dashmap::DashMap;
use std::sync::Arc;

use crate::{
    config::AppConfig,
    error::{BotError, BotResult},
    state::AppState,
};

#[derive(Clone)]
pub struct LastInterfaceState {
    pub last_access: DateTime<Utc>,
    pub interface: String,
}

impl Default for LastInterfaceState {
    fn default() -> Self {
        Self {
            last_access: Utc::now(),
            interface: "main".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct InteractionService {
    // pub interactions: Arc<DashMap<String, DateTime<Utc>>>,
    pub interfaces: Arc<DashMap<String, LastInterfaceState>>,
    // cooldown: TimeDelta,
    interface_lifespan: TimeDelta,
}

impl InteractionService {
    pub fn new() -> BotResult<Self> {
        info!("Initializing InteractionService...");
        let config = AppConfig::get()?;
        let interfaces = Arc::new(DashMap::with_capacity(config.interaction.cache_capacity));
        // let interactions = Arc::new(DashMap::with_capacity(config.interaction.cache_capacity));
        // let cooldown = Duration::milliseconds(config.interaction.cooldown_ms);
        let interface_lifespan = Duration::seconds(config.interaction.interface_lifespan_secs);

        info!("InteractionService initialized");
        Ok(Self {
            interfaces,
            // interactions,
            // cooldown,
            interface_lifespan,
        })
    }

    // pub fn should_process(&self, user_id: String) -> bool {
    //     let now = Utc::now();

    //     // Use entry API to atomically check and update
    //     match self.interactions.entry(user_id) {
    //         dashmap::mapref::entry::Entry::Occupied(mut entry) => {
    //             let last_time = entry.get();
    //             let time_since_last = (now - last_time).num_milliseconds();
    //             let cooldown_ms = self.cooldown.num_milliseconds();

    //             info!(
    //                 "Interaction check - Now: {:?}, Last: {:?}, Diff: {}ms, Cooldown: {}ms",
    //                 now, last_time, time_since_last, cooldown_ms
    //             );

    //             // If within cooldown period, reject the interaction
    //             if time_since_last < cooldown_ms {
    //                 info!(
    //                     "Interaction throttled - Time since last: {}ms < Cooldown: {}ms",
    //                     time_since_last, cooldown_ms
    //                 );
    //                 return false;
    //             }

    //             // Update the entry if we're allowing the interaction
    //             entry.insert(now);
    //             true
    //         }
    //         dashmap::mapref::entry::Entry::Vacant(entry) => {
    //             // First interaction for this user
    //             entry.insert(now);
    //             true
    //         }
    //     }
    // }

    // Interface tracking methods
    pub fn set_last_interface(&self, telegram_user_id: String, interface: &str) {
        let now = Utc::now();
        self.interfaces.insert(
            telegram_user_id,
            LastInterfaceState {
                last_access: now,
                interface: interface.to_string(),
            },
        );
    }

    pub fn get_last_interface(&self, telegram_user_id: String) -> Option<LastInterfaceState> {
        self.interfaces.get(&telegram_user_id).map(|v| v.clone())
    }

    /// Background task to cleanup old interfaces
    pub fn cleanup_old_entries(&self) {
        let now = Utc::now();
        // Cleanup old interactions (after cooldown period)
        // self.interactions
        //     .retain(|_, last_time| now - *last_time < self.cooldown);
        // Cleanup old interfaces
        self.interfaces
            .retain(|_, state| now - state.last_access < self.interface_lifespan);
    }
    /// Background task to save interfaces to database
    // pub async fn save_interface_to_database(
    //     &self,
    //     telegram_user_id: &str,
    //     interface: LastInterfaceState,
    // ) -> BotResult<()> {
    //     let app_state = AppState::get()?;
    //     let conn = app_state.turso.get_connection().await?;

    //     let interface_str = format!("{:?}:{:?}", interface.interface, interface.last_access);
    //     conn.execute(
    //         "INSERT OR REPLACE INTO user_last_interface (telegram_user_id, interface) VALUES (?1, ?2)",
    //         params![telegram_user_id, interface_str],
    //     )
    //     .await
    //     .map_err(|e| BotError::TursoError(e.to_string()))?;
    //     Ok(())
    // }

    pub async fn save_interfaces_to_database(&self) -> BotResult<()> {
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

        for entry in self.interfaces.iter() {
            let telegram_user_id = entry.key();
            let interface = entry.value();
            let interface_str = format!("{:?}:{:?}", interface.interface, interface.last_access);

            values.push(format!("(?{}, ?{})", params.len() + 1, params.len() + 2));
            params.push(telegram_user_id.clone());
            params.push(interface_str);
        }

        if !values.is_empty() {
            // Build and execute batch query
            let query = format!(
                "INSERT OR REPLACE INTO user_last_interface (telegram_user_id, interface) VALUES {}",
                values.join(",")
            );

            tx.execute(&query, params)
                .await
                .map_err(|e| BotError::TursoError(e.to_string()))?;
        }

        // Commit transaction
        tx.commit().await.map_err(|e| BotError::TursoError(e.to_string()))?;

        // TODO: Turso error: Hrana: `stream error: `Error { message: "SQLite error: database is locked", code: "SQLITE_BUSY" }``
        Ok(())
    }
}
