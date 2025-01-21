use crate::error::{BotError, BotResult};
use libsql::{Builder, Connection, Database};
use std::sync::Arc;

use super::StorageError;

#[derive(Clone)]
pub struct TursoClient {
    inner: Arc<Database>,
}

impl TursoClient {
    pub async fn new(url: &str, token: &str) -> Result<Self, StorageError> {
        info!("Initializing TursoClient...");
        let db = Arc::new(
            Builder::new_remote(url.to_string(), token.to_string())
                .build()
                .await
                .map_err(|e| StorageError::Turso(e))?,
        );

        info!("TursoClient initialized");
        Ok(Self { inner: db })
    }

    pub async fn get_connection(&self) -> Result<Connection, StorageError> {
        let conn = self.inner.connect().map_err(|e| StorageError::Turso(e))?;
        Ok(conn)
    }
}
