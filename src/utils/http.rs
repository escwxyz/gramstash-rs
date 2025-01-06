use reqwest::{
    cookie::Jar,
    header::{self, HeaderMap, HeaderValue},
    Client,
};
use std::{sync::Arc, time::Duration};

pub fn create_telegram_client() -> Client {
    let builder = Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(60))
        .tcp_keepalive(Duration::from_secs(30))
        .user_agent("TelegramBot/1.0");

    build_client(builder)
}

pub fn create_instagram_client(cookie_store: Arc<Jar>) -> Client {
    let mut headers = HeaderMap::new();
    headers.insert(header::ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(header::ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
    headers.insert("X-IG-App-ID", HeaderValue::from_static("936619743392459"));
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );
    headers.insert(header::ORIGIN, HeaderValue::from_static("https://www.instagram.com"));
    headers.insert(header::REFERER, HeaderValue::from_static("https://www.instagram.com/"));
    headers.insert(
        "Sec-Ch-Ua",
        HeaderValue::from_static("\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\""),
    );
    headers.insert("Sec-Ch-Ua-Mobile", HeaderValue::from_static("?0"));
    headers.insert("Sec-Ch-Ua-Platform", HeaderValue::from_static("\"Windows\""));

    let builder = Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(30))
        .cookie_provider(Arc::clone(&cookie_store))
        .default_headers(headers)
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");

    build_client(builder)
}

fn build_client(builder: reqwest::ClientBuilder) -> Client {
    #[cfg(debug_assertions)]
    {
        info!("Debug mode: configuring client with proxy");
        let proxy_url = "socks5://127.0.0.1:1080";
        builder
            .proxy(reqwest::Proxy::all(proxy_url).expect("Failed to create proxy"))
            .build()
            .expect("Failed to build client with proxy")
    }

    #[cfg(not(debug_assertions))]
    {
        builder.build().expect("Failed to build client")
    }
}
