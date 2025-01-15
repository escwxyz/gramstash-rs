use reqwest::header::{self, HeaderMap, HeaderValue};

use crate::error::BotResult;

pub const DEFAULT_USER_AGENT: &str = "TelegramBot/1.0";
pub const INSTAGRAM_USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

pub fn build_client(builder: reqwest::ClientBuilder) -> BotResult<reqwest::Client> {
    info!("Building client ...");

    let client = if cfg!(debug_assertions) {
        info!("Debug mode: configuring client with proxy");
        let proxy_url = "socks5://127.0.0.1:1080";
        builder
            .proxy(reqwest::Proxy::all(proxy_url).map_err(|_| anyhow::anyhow!("Failed to create proxy"))?) // TODO
            .build()
            .map_err(|_| anyhow::anyhow!("Failed to build client with proxy"))?
    } else {
        builder.build().map_err(|_| anyhow::anyhow!("Failed to build client"))?
    };

    Ok(client)
}

pub fn build_desktop_instagram_headers() -> HeaderMap {
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
    headers.insert("sec-ch-prefers-color-scheme", HeaderValue::from_static("dark"));
    headers.insert(
        "Sec-Ch-Ua",
        HeaderValue::from_static("\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\""),
    );
    headers.insert("Sec-Ch-Ua-Mobile", HeaderValue::from_static("?0"));
    headers.insert("Sec-Ch-Ua-Platform", HeaderValue::from_static("\"macOS\""));

    headers
}
