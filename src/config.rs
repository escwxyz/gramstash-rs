#[derive(Clone, Debug)]
pub struct AppConfig {
    pub redis: RedisConfig,
    pub telegram: TelegramConfig,
    pub instagram: InstagramConfig,
    pub rate_limit: RateLimitConfig,
    pub cache: CacheConfig,
    pub dialogue: DialogueConfig,
}

#[derive(Clone, Debug)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct TelegramConfig(pub String);

#[derive(Clone, Debug)]
pub struct InstagramConfig {
    pub api_endpoint: String,
    pub doc_id: String,
}

#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    pub daily_limit: u32,
    pub window_secs: u64,
}

#[derive(Clone, Debug)]
pub struct CacheConfig {
    pub expiry_secs: u64,
}

#[derive(Clone, Debug)]
pub struct DialogueConfig {
    pub use_redis: bool,
    pub clear_interval_secs: u64,
}
