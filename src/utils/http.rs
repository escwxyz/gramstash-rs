use reqwest::Client;
use std::time::Duration;

pub fn create_client(timeout: Duration) -> Client {
    #[cfg(debug_assertions)]
    {
        info!("Debug mode: configuring client with proxy");
        let proxy_url = "socks5://127.0.0.1:1080";
        Client::builder()
            .proxy(reqwest::Proxy::all(proxy_url).expect("Failed to create proxy"))
            .timeout(timeout)
            .connect_timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .expect("Failed to build client with proxy")
    }

    #[cfg(not(debug_assertions))]
    {
        Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to build client")
    }
}

// Optional: Create with default timeouts
pub fn create_default_client() -> Client {
    create_client(Duration::from_secs(30))
}

// Optional: Create with extended timeouts (useful for downloads)
pub fn create_download_client() -> Client {
    create_client(Duration::from_secs(60))
}
