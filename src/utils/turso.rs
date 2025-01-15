use crate::error::{BotError, BotResult};
use libsql::{Builder, Connection, Database};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TursoClient(pub Arc<Database>);

impl TursoClient {
    pub async fn new(url: &str, token: &str) -> BotResult<Self> {
        let db = Arc::new(
            Builder::new_remote(url.to_string(), token.to_string())
                .build()
                .await
                .map_err(|e| BotError::TursoError(format!("Failed to connect to Turso database: {}", e)))?,
        );

        info!("Connected to Turso database");
        Ok(Self(db))
    }

    pub async fn get_connection(&self) -> BotResult<Connection> {
        let conn = self.0.connect().map_err(|e| BotError::TursoError(e.to_string()))?;
        Ok(conn)
    }
}
