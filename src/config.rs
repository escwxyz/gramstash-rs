use shuttle_runtime::SecretStore;

#[derive(Debug)]
pub struct Config {
    pub telegram_token: String,
    pub redis_url: String,
}

impl Config {
    pub fn get(secret_store: &SecretStore) -> Self {

        let telegram_token = secret_store.get("TELEGRAM_BOT_TOKEN").expect("Failed to get the telegram token!");
        let redis_host = secret_store.get("REDIS_HOST").expect("Failed to get the redis host!");
        let redis_port = secret_store.get("REDIS_PORT").expect("Failed to get the redis port!");
        let redis_password = secret_store.get("REDIS_PASSWORD").expect("Failed to get the redis password!");

        let redis_url = format!("redis://:{}@{}:{}", redis_password, redis_host, redis_port);

        Self {
            telegram_token,
            redis_url,
        }
    }
}