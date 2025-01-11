use redis::{aio::MultiplexedConnection, Client};
use std::sync::Arc;

use crate::error::{BotError, BotResult};

#[derive(Clone)]
pub struct RedisClient(pub Arc<Client>);

impl RedisClient {
    pub async fn new(url: &str) -> BotResult<Self> {
        let redis = Arc::new(Client::open(url)?);

        // Test Redis connection
        let mut conn = redis.get_multiplexed_async_connection().await?;
        let pong: String = redis::cmd("PING").query_async(&mut conn).await?;
        if pong != "PONG" {
            return Err(BotError::RedisError("Redis connection test failed".to_string()));
        }
        info!("Redis connection test successful");

        Ok(Self(redis))
    }

    pub async fn get_connection(&self) -> BotResult<MultiplexedConnection> {
        let conn = self.0.get_multiplexed_async_connection().await?;
        Ok(conn)
    }
}
