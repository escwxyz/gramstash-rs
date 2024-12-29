#[derive(Clone, Debug)]
pub struct AppConfig {
    pub redis: RedisConfig,
    pub telegram: TelegramConfig,
    pub instagram: InstagramConfig,
    pub rate_limit: RateLimitConfig,
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
