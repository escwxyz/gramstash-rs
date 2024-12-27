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

use super::instagram::InstagramService;

#[derive(Clone)]
pub struct DownloaderService {
    client: Client,
    redis: redis::Client,
    storage_path: PathBuf,
    rate_limiter: RateLimiter,
    pub instagram_service: InstagramService,
}

#[derive(Clone)]
struct RateLimiter {
    redis: redis::Client,
}

impl RateLimiter {
    pub fn new(redis_url: &str) -> Result<Self> {
        let redis = redis::Client::open(redis_url)?;
        Ok(Self { redis })
    }

    pub async fn check_limit(&self, chat_id: i64) -> Result<(), BotError> {
        let mut conn = self
            .redis
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| BotError::NetworkError(format!("Redis connection error: {}", e)))?;

        let key = format!("rate_limit:{}:{}", chat_id, chrono::Utc::now().date_naive());

        let count: Option<i32> = conn
            .get(&key)
            .await
            .map_err(|e| BotError::NetworkError(format!("Redis error: {}", e)))?;

        if count.unwrap_or(0) >= 1 {
            return Err(BotError::ApiError(
                "Daily download limit reached. Try again tomorrow!".into(),
            ));
        }

        // Set with 24-hour expiry
        let _: () = redis::pipe()
            .atomic()
            .incr(&key, 1)
            .expire(&key, 24 * 3600)
            .query_async(&mut conn)
            .await
            .map_err(|e| BotError::NetworkError(format!("Redis error: {}", e)))?;

        Ok(())
    }
}

impl DownloaderService {
    pub async fn new(
        storage_path: PathBuf,
        redis_url: &str,
        instagram_api_endpoint: String,
        instagram_doc_id: String,
    ) -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        info!("Initializing Redis client...");

        let redis = redis::Client::open(redis_url)?;

        info!("Redis client initialized");
        // Create storage directory if it doesn't exist
        tokio::fs::create_dir_all(&storage_path).await?;

        info!("Initializing Instagram service...");

        // Configure Instagram client with proxy in debug mode
        #[cfg(debug_assertions)]
        let instagram_client = {
            info!("Debug mode: configuring Instagram client with proxy");
            let proxy_url = "socks5://127.0.0.1:1080";
            Client::builder()
                .proxy(reqwest::Proxy::all(proxy_url).expect("Failed to create proxy"))
                .timeout(Duration::from_secs(60))
                .connect_timeout(Duration::from_secs(30))
                .pool_idle_timeout(Duration::from_secs(90))
                .tcp_keepalive(Duration::from_secs(60))
                .build()
                .expect("Failed to build Instagram client with proxy")
        };

        // Use regular client in release mode
        #[cfg(not(debug_assertions))]
        let instagram_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build Instagram client");

        let instagram_service =
            InstagramService::new(instagram_client.clone(), instagram_api_endpoint, instagram_doc_id);

        Ok(Self {
            client,
            redis,
            storage_path,
            rate_limiter: RateLimiter::new(redis_url)?,
            instagram_service,
        })
    }

    pub async fn download_media(&self, url: &str, chat_id: i64) -> Result<PathBuf, BotError> {
        // Check rate limit first
        self.rate_limiter.check_limit(chat_id).await?;

        // Generate a unique filename based on URL and timestamp
        // TODO: test it
        let file_hash = {
            let mut hasher = Sha256::new();
            hasher.update(url.as_bytes());
            hasher.update(chrono::Utc::now().timestamp().to_string().as_bytes());
            format!("{:x}", hasher.finalize())
        };

        // Check if we already have this file cached
        let cache_key = format!("media_cache:{}", file_hash);
        let mut redis_conn = self
            .redis
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| BotError::NetworkError(format!("Redis error: {}", e)))?;

        if let Ok(cached_path) = redis_conn.get::<_, String>(&cache_key).await {
            let cached_path = PathBuf::from(cached_path);
            if cached_path.exists() {
                return Ok(cached_path);
            }
        }

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
        let _: () = redis_conn
            .set_ex(&cache_key, file_path.to_str().unwrap(), 24 * 3600)
            .await
            .map_err(|e| BotError::NetworkError(format!("Redis error: {}", e)))?;

        Ok(file_path)
    }
}
