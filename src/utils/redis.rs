use anyhow::{Context, Result};
use redis::AsyncCommands;
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

    // pub async fn get_cached<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, anyhow::Error> {
    //     let mut conn = self.get_connection().await?;
    //     let data: Option<String> = redis::cmd("GET").arg(key).query_async(&mut conn).await?;
    //     match data {
    //         Some(json) => Ok(Some(serde_json::from_str(&json)?)),
    //         None => Ok(None),
    //     }
    // }

    pub async fn set_cached<T: serde::Serialize>(&self, key: &str, value: &T, expiry_secs: u64) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let json = serde_json::to_string(value)?;
        conn.set_ex::<_, _, u64>(key, json, expiry_secs).await?;

        Ok(())
    }

    // pub async fn incr_rate_limit(&self, key: &str) -> Result<(), anyhow::Error> {
    //     let mut conn = self.get_connection().await?;
    //     conn.incr::<_, u32, u32>(key, 1).await?;
    //     Ok(())
    // }

    // pub async fn expire_rate_limit(&self, key: &str, expiry_secs: u64) -> Result<(), anyhow::Error> {
    //     let mut conn = self.get_connection().await?;
    //     conn.expire::<_, i64>(key, expiry_secs as i64).await?;
    //     Ok(())
    // }
}
