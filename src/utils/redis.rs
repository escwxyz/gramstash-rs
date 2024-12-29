use anyhow::{Context, Result};
use redis::{aio::MultiplexedConnection, Client};
use std::sync::Arc;

#[derive(Clone)]
pub struct RedisClient(pub Arc<Client>);

impl RedisClient {
    pub async fn new(url: &str) -> Result<Self> {
        let redis = Arc::new(Client::open(url)?);

        // Test Redis connection
        let mut conn = redis
            .get_multiplexed_async_connection()
            .await
            .context("Failed to get Redis connection")?;
        let pong: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .context("Failed to ping Redis")?;
        if pong != "PONG" {
            return Err(anyhow::anyhow!("Redis connection test failed"));
        }
        info!("Redis connection test successful");

        Ok(Self(redis))
    }

    pub async fn get_connection(&self) -> Result<MultiplexedConnection> {
        let conn = self
            .0
            .get_multiplexed_async_connection()
            .await
            .context("Failed to get Redis connection")?;
        Ok(conn)
    }
}
