// TODO: implement this

use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use reqwest::Client;
use std::time::Duration;
use redis::{AsyncCommands, RedisResult};
use sha2::{Sha256, Digest};
use anyhow::Result;

pub struct DownloaderService {
    client: Client,
    redis: redis::Client,
    storage_path: PathBuf,
    rate_limiter: RateLimiter,
}

struct RateLimiter {
    redis: redis::Client,
}

impl DownloaderService {
    pub fn new(storage_path: PathBuf, redis_url: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        let redis = redis::Client::open(redis_url)?;

        Ok(Self {
            client,
            redis,
            storage_path,
            rate_limiter: RateLimiter::new(redis_url)?,
        })
    }

    pub async fn download_media(&self, url: &str, chat_id: i64) -> Result<PathBuf> {
        // Check rate limit
        // TODO: implement this
        // self.rate_limiter.check_limit(chat_id).await?;

        // Generate unique filename from URL
        let file_hash = self.generate_file_hash(url);
        let file_path = self.storage_path.join(&file_hash);

        // Check if file exists in cache
        if file_path.exists() {
            return Ok(file_path);
        }

        // Download file
        let response = self.client.get(url).send().await?;
        let content = response.bytes().await?;

        // Create directory if it doesn't exist
        fs::create_dir_all(&self.storage_path).await?;

        // Save file
        let mut file = fs::File::create(&file_path).await?;
        file.write_all(&content).await?;

        // Store metadata in Redis
        // TODO: implement this
        // self.store_metadata(&file_hash, url, chat_id).await?;

        Ok(file_path)
    }

    fn generate_file_hash(&self, url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    // TODO: implement this
    // async fn store_metadata(&self, file_hash: &str, url: &str, chat_id: i64) -> RedisResult<()> {

    //     let mut conn = self.redis.get_multiplexed_async_connection().await?;
        
    //     // Store URL -> hash mapping
    //     conn.set_ex(
    //         format!("url:{}", url),
    //         file_hash,
    //         24 * 3600, // 24 hours TTL
    //     ).await?;

    //     // Store download info
    //     conn.hset(
    //         format!("file:{}", file_hash),
    //         &[
    //             ("url", url),
    //             ("chat_id", &chat_id.to_string()),
    //             ("timestamp", &chrono::Utc::now().timestamp().to_string()),
    //         ],
    //     ).await?;

    //     Ok(())
    // }

    // TODO: implement this

    // pub async fn cleanup_old_files(&self, max_age_hours: u64) -> Result<()> {
    //     let cutoff = chrono::Utc::now() - chrono::Duration::hours(max_age_hours as i64);
        
    //     let mut conn = self.redis.get_multiplexed_async_connection().await?;
    //     let files: Vec<String> = conn.keys("file:*").await?;

    //     for file_key in files {
    //         let timestamp: i64 = conn.hget(&file_key, "timestamp").await?;
    //         if timestamp < cutoff.timestamp() {
    //             // Delete file and metadata
    //             let file_hash = file_key.trim_start_matches("file:");
    //             let file_path = self.storage_path.join(file_hash);
                
    //             if file_path.exists() {
    //                 fs::remove_file(&file_path).await?;
    //             }
    //             conn.del(&file_key).await?;
    //         }
    //     }

    //     Ok(())
    // }
}

impl RateLimiter {
    pub fn new(redis_url: &str) -> Result<Self> {
        let redis = redis::Client::open(redis_url)?;
        Ok(Self { redis })
    }

    // TODO: implement this
    // async fn check_limit(&self, chat_id: i64) -> Result<()> {
    //     let mut conn = self.redis.get_multiplexed_async_connection().await?;
    //     let key = format!("rate:{}:{}", chat_id, chrono::Utc::now().date_naive());
        
    //     let count: Option<i32> = conn.get(&key).await?;
    //     let count = count.unwrap_or(0);

    //     if count >= 50 { // 50 downloads per day limit
    //         anyhow::bail!("Rate limit exceeded. Try again tomorrow!");
    //     }

    //     conn.incr(&key, 1).await?;
    //     conn.expire(&key, 24 * 3600).await?; // 24 hours TTL

    //     Ok(())
    // }
}