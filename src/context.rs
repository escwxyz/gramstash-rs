use std::sync::{Arc, OnceLock};

use serde::{Deserialize, Serialize};
use teloxide::types::UserId;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct UserContext {
    pub telegram_user_id: OnceLock<UserId>,
    pub telegram_user_name: OnceLock<String>,
    pub is_admin: OnceLock<bool>,
    user_tier: Arc<Mutex<UserTier>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
pub enum UserTier {
    Subscriber = 3,
    OneTimePaid = 2,
    Free = 1,
}

impl UserContext {
    pub fn new() -> Self {
        Self {
            telegram_user_id: OnceLock::new(),
            telegram_user_name: OnceLock::new(),
            is_admin: OnceLock::new(),
            user_tier: Arc::new(Mutex::new(UserTier::Free)),
        }
    }

    pub fn init(&self, user_id: UserId, user_name: String, is_admin: bool) {
        let _ = self.telegram_user_id.set(user_id);
        let _ = self.telegram_user_name.set(user_name);
        let _ = self.is_admin.set(is_admin);
    }

    pub fn user_id(&self) -> UserId {
        *self.telegram_user_id.get().expect("UserContext not initialized")
    }

    pub fn user_name(&self) -> &str {
        self.telegram_user_name.get().expect("UserContext not initialized")
    }

    pub fn is_admin(&self) -> bool {
        *self.is_admin.get().expect("UserContext not initialized")
    }

    pub async fn user_tier(&self) -> UserTier {
        self.user_tier.lock().await.clone()
    }

    // pub async fn set_user_tier(&self, user_tier: UserTier) {
    //     *self.user_tier.lock().await = user_tier;
    // }
}

static USER_DATA: OnceLock<UserContext> = OnceLock::new();

impl UserContext {
    pub fn global() -> &'static UserContext {
        USER_DATA.get_or_init(UserContext::new)
    }

    pub async fn ensure_initialized(user_id: UserId, user_name: String, is_admin: bool) {
        let user_data = Self::global();
        user_data.init(user_id, user_name, is_admin);
    }
}
