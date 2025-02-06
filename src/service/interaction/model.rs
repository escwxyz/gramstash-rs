use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
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
