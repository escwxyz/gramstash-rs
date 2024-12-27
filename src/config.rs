use shuttle_runtime::SecretStore;

#[derive(Debug)]
pub struct Config {
    pub telegram_token: String,
    pub redis_url: String,
    pub instagram_api_endpoint: String,
    pub instagram_doc_id: String,
}

impl Config {
    pub fn get(secret_store: &SecretStore) -> Self {
        let telegram_token = secret_store
            .get("TELEGRAM_BOT_TOKEN")
            .expect("Failed to get the telegram token!");
        let redis_host = secret_store
            .get("UPSTASH_REDIS_HOST")
            .expect("Failed to get the redis host!");
        let redis_port = secret_store
            .get("UPSTASH_REDIS_PORT")
            .expect("Failed to get the redis port!");
        let redis_password = secret_store
            .get("UPSTASH_REDIS_PASSWORD")
            .expect("Failed to get the redis password!");

        let instagram_api_endpoint = secret_store
            .get("INSTAGRAM_API_ENDPOINT")
            .expect("Failed to get the instagram api endpoint!");
        let instagram_doc_id = secret_store
            .get("INSTAGRAM_DOC_ID")
            .expect("Failed to get the instagram doc id!");

        let redis_url = format!("redis://default:{}@{}:{}", redis_password, redis_host, redis_port);

        Self {
            telegram_token,
            redis_url,
            instagram_api_endpoint,
            instagram_doc_id,
        }
    }
}
