// TODO: implement this

use anyhow::Result;
use redis::AsyncCommands;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::utils::error::BotError;
use crate::utils::http;

use super::instagram::InstagramService;

#[derive(Clone)]
pub struct DownloaderService {
    client: Client,
    // redis_connection: redis::aio::MultiplexedConnection,
    storage_path: PathBuf,
    // rate_limiter: RateLimiter,
    pub instagram_service: InstagramService,
}

#[derive(Clone)]
struct RateLimiter {
    redis_connection: redis::aio::MultiplexedConnection,
}

impl RateLimiter {
    pub fn new(connection: redis::aio::MultiplexedConnection) -> Self {
        info!("Initializing RateLimiter with Redis connection");
        Self { 
            redis_connection: connection 
        }
    }

    pub async fn check_limit(&self, chat_id: i64) -> Result<(), BotError> {
        info!("Checking rate limit for chat_id: {}", chat_id);
        
        let mut conn = self.redis_connection.clone();
        let key = format!("rate_limit:{}:{}", chat_id, chrono::Utc::now().date_naive());
        info!("Using Redis key: {}", key);

        let count: Option<i32> = conn
            .get(&key)
            .await
            .map_err(|e| {
                error!("Redis get error: {:?}", e);
                BotError::NetworkError(format!("Redis error: {}", e))
            })?;

        info!("Current count for {}: {:?}", key, count);

        if count.unwrap_or(0) >= 1 {
            return Err(BotError::ApiError(
                "Daily download limit reached. Try again tomorrow!".into(),
            ));
        }

        // Set with 24-hour expiry using pipeline
        info!("Setting rate limit with 24h expiry");
        let result: Result<(), redis::RedisError> = redis::pipe()
            .atomic()
            .incr(&key, 1)
            .expire(&key, 24 * 3600)
            .query_async(&mut conn)
            .await;

        if let Err(e) = result {
            error!("Redis pipeline error: {:?}", e);
            return Err(BotError::NetworkError(format!("Redis error: {}", e)));
        }

        info!("Rate limit check completed successfully");
        Ok(())
    }
}

impl DownloaderService {
    pub async fn new(
        storage_path: PathBuf,
        // redis_url: &str,
        instagram_api_endpoint: String,
        instagram_doc_id: String,
    ) -> Result<Self> {
        info!("Initializing DownloaderService");
        
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
        info!("HTTP client initialized");

        // Initialize Redis connection with retries
        info!("Initializing Redis connection...");
        // let redis_connection = Self::init_redis_connection(redis_url).await?;
        info!("Redis connection established successfully");

        // Create storage directory if it doesn't exist
        tokio::fs::create_dir_all(&storage_path).await?;
        info!("Storage directory ensured: {:?}", storage_path);

        // let rate_limiter = RateLimiter::new(redis_connection.clone());
        info!("Rate limiter initialized");

        let instagram_client = http::create_download_client();

        let instagram_service = InstagramService::new(
            instagram_client,
            instagram_api_endpoint,
            instagram_doc_id,
        );

        Ok(Self {
            client,
            // redis_connection,
            storage_path,
            // rate_limiter,
            instagram_service,
        })
    }

    // async fn init_redis_connection(redis_url: &str) -> Result<redis::aio::MultiplexedConnection, BotError> {
    //     let max_retries = 3;
    //     let mut last_error = None;

    //     for attempt in 1..=max_retries {
    //         info!("Attempting to establish Redis connection (attempt {}/{})", attempt, max_retries);
            
    //         let redis_client = match redis::Client::open(redis_url) {
    //             Ok(client) => client,
    //             Err(e) => {
    //                 error!("Failed to create Redis client: {:?}", e);
    //                 continue;
    //             }
    //         };

    //         match redis_client.get_multiplexed_async_connection().await {
    //             Ok(mut conn) => {
    //                 // Test the connection by sending a PING command
    //                 match redis::cmd("PING").query_async(&mut conn).await {
    //                     Ok(_) => {
    //                         info!("Redis connection test successful");
    //                         return Ok(conn);
    //                     }
    //                     Err(e) => {
    //                         error!("Redis ping test failed: {:?}", e);
    //                         last_error = Some(e.to_string());
    //                     }
    //                 }
    //             }
    //             Err(e) => {
    //                 error!("Failed to get Redis connection: {:?}", e);
    //                 last_error = Some(e.to_string());
    //             }
    //         }

    //         if attempt < max_retries {
    //             let delay = Duration::from_secs(2);
    //             info!("Waiting {:?} before retry", delay);
    //             tokio::time::sleep(delay).await;
    //         }
    //     }

    //     Err(BotError::NetworkError(format!(
    //         "Failed to establish Redis connection after {} attempts. Last error: {}",
    //         max_retries,
    //         last_error.unwrap_or_else(|| "Unknown error".to_string())
    //     )))
    // }

    pub async fn download_media(&self, url: &str, chat_id: i64) -> Result<PathBuf, BotError> {
        // Check rate limit first
        // self.rate_limiter.check_limit(chat_id).await?;

        // Generate a unique filename based on URL and timestamp
        // TODO: test it
        let file_hash = {
            let mut hasher = Sha256::new();
            hasher.update(url.as_bytes());
            hasher.update(chrono::Utc::now().timestamp().to_string().as_bytes());
            format!("{:x}", hasher.finalize())
        };

        // Check if we already have this file cached
        // TODO: implement this
        // let cache_key = format!("media_cache:{}", file_hash);
        // let mut redis_conn = self.redis_connection.clone();

        // if let Ok(cached_path) = redis_conn.get::<_, String>(&cache_key).await {
        //     let cached_path = PathBuf::from(cached_path);
        //     if cached_path.exists() {
        //         return Ok(cached_path);
        //     }
        // }

        // If the file is not cached, download it
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| BotError::NetworkError(format!("Failed to download: {}", e)))?;

        // Get content type and extension
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("application/octet-stream");

        let extension = if content_type.contains("image") {
            "jpg"
        } else if content_type.contains("video") {
            "mp4"
        } else {
            "bin"
        };

        // Create file path
        let file_path = self.storage_path.join(format!("{}.{}", file_hash, extension));

        // Download and save the file
        let bytes = response
            .bytes()
            .await
            .map_err(|e| BotError::NetworkError(format!("Failed to read response: {}", e)))?;

        let mut file = fs::File::create(&file_path)
            .await
            .map_err(|e| BotError::NetworkError(format!("Failed to create file: {}", e)))?;

        file.write_all(&bytes)
            .await
            .map_err(|e| BotError::NetworkError(format!("Failed to write file: {}", e)))?;

        // Cache the file path in Redis (with 24-hour expiry)
        // TODO: implement this
        // let _: () = redis_conn
        //     .set_ex(&cache_key, file_path.to_str().unwrap(), 24 * 3600)
        //     .await
        //     .map_err(|e| BotError::NetworkError(format!("Redis error: {}", e)))?;

        Ok(file_path)
    }
}
