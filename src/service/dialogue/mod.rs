use std::sync::Arc;

use model::DialogueState;
use teloxide::dispatching::dialogue::{serializer::Json, ErasedStorage, RedisStorage, Storage};

use crate::{config::StorageConfig, storage::StorageError};

use super::ServiceError;

pub mod model;

pub struct DialogueService;

impl DialogueService {
    pub async fn get_dialogue_storage(
        config: &StorageConfig,
    ) -> Result<Arc<ErasedStorage<DialogueState>>, ServiceError> {
        let storage = RedisStorage::open(config.redis_url.as_str(), Json)
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?
            .erase();

        Ok(storage)
    }
}
