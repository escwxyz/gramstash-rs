use anyhow::Context;
use redis::{aio::MultiplexedConnection, Client};
use std::sync::Arc;

use super::error::BotResult;
use crate::utils::error::BotError;

#[derive(Clone)]
pub struct RedisClient(pub Arc<Client>);

impl RedisClient {
    pub async fn new(url: &str) -> BotResult<Self> {
        let redis = Arc::new(Client::open(url).context("Failed to open Redis connection")?);

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
            return Err(BotError::RedisError("Redis connection test failed".to_string()));
        }
        info!("Redis connection test successful");

        Ok(Self(redis))
    }

    pub async fn get_connection(&self) -> BotResult<MultiplexedConnection> {
        let conn = self
            .0
            .get_multiplexed_async_connection()
            .await
            .context("Failed to get Redis connection")?;
        Ok(conn)
    }
}
