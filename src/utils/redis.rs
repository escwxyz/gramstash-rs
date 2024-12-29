use anyhow::Result;
use redis::{aio::MultiplexedConnection, Client};
use std::sync::Arc;

#[derive(Clone)]
pub struct RedisClient(pub Arc<Client>);

impl RedisClient {
    pub async fn new(url: &str) -> Result<Self> {
        let redis = Arc::new(Client::open(url)?);

        // Test Redis connection
        let mut conn = redis.get_multiplexed_async_connection().await?;
        let pong: String = redis::cmd("PING").query_async(&mut conn).await?;
        if pong != "PONG" {
            return Err(anyhow::anyhow!("Redis connection test failed"));
        }
        info!("Redis connection test successful");

        Ok(Self(redis))
    }

    pub async fn get_connection(&self) -> Result<MultiplexedConnection, anyhow::Error> {
        let conn = self.0.get_multiplexed_async_connection().await?;
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
}

// pub async fn get_connection() -> Result<MultiplexedConnection, anyhow::Error> {
//     let state = AppState::get();

//     state
//         .redis
//         .get_multiplexed_async_connection()
//         .await
//         .map_err(|e| anyhow::anyhow!("Failed to get Redis connection: {}", e))
// }

// pub async fn get_cached<T: serde::de::DeserializeOwned>(key: &str) -> Result<Option<T>, anyhow::Error> {
//     let mut conn = get_connection().await?;

//     let data: Option<String> = redis::cmd("GET").arg(key).query_async(&mut conn).await?;

//     match data {
//         Some(json) => Ok(Some(serde_json::from_str(&json)?)),
//         None => Ok(None),
//     }
// }

// pub async fn set_cached<T: serde::Serialize>(
//     key: &str,
//     value: &T,
//     expiry_secs: u64
// ) -> Result<(), anyhow::Error> {
//     let mut conn = get_connection().await?;
//     let json = serde_json::to_string(value)?;

//     redis::cmd("SETEX")
//         .arg(key)
//         .arg(expiry_secs)
//         .arg(json)
//         .query_async(&mut conn)
//         .await?;

//     Ok(())
// }
