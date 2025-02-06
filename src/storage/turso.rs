use libsql::errors::Error as TursoError;
use libsql::{Builder, Connection, Database};
use std::sync::{Arc, OnceLock};

use super::StorageError;

pub static TURSO_CLIENT: OnceLock<TursoClient> = OnceLock::new();

#[derive(Clone)]
pub struct TursoClient {
    inner: Arc<Database>,
}

impl TursoClient {
    pub async fn init(url: &str, token: &str) -> Result<(), StorageError> {
        if TURSO_CLIENT.get().is_some() {
            info!("TursoClient already initialized");
            return Ok(());
        }

        info!("Initializing TursoClient...");
        let db = Arc::new(
            Builder::new_remote(url.to_string(), token.to_string())
                .build()
                .await
                .map_err(|e| StorageError::Turso(e))?,
        );

        info!("TursoClient initialized");
        TURSO_CLIENT.set(Self { inner: db }).map_err(|_| {
            StorageError::Turso(TursoError::ConnectionFailed(
                "Failed to set global Turso client".to_string(),
            ))
        })?;

        Ok(())
    }

    pub fn get() -> Result<&'static TursoClient, StorageError> {
        TURSO_CLIENT.get().ok_or_else(|| {
            StorageError::Turso(TursoError::ConnectionFailed("Turso client not initialized".to_string()))
        })
    }

    // pub async fn new(url: &str, token: &str) -> Result<Self, StorageError> {
    //     info!("Initializing TursoClient...");
    //     let db = Arc::new(
    //         Builder::new_remote(url.to_string(), token.to_string())
    //             .build()
    //             .await
    //             .map_err(|e| StorageError::Turso(e))?,
    //     );

    //     info!("TursoClient initialized");
    //     Ok(Self { inner: db })
    // }

    pub async fn get_connection(&self) -> Result<Connection, StorageError> {
        let conn = self.inner.connect().map_err(|e| StorageError::Turso(e))?;
        Ok(conn)
    }
}
