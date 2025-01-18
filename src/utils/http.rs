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

    info!("Client built");
    Ok(client)
}

pub fn build_desktop_instagram_headers(anonymous: bool) -> HeaderMap {
    let mut headers = HeaderMap::new();

    // Standard headers
    headers.insert(header::ACCEPT, HeaderValue::from_static("*/*"));
    // NOTE: Don't set ACCEPT_ENCODING header
    // Let reqwest handle compression automatically with its default supported algorithms
    // If we specify additional compression types like 'zstd' without having a decoder,
    // we'll get garbled responses
    //
    // headers.insert(
    //     header::ACCEPT_ENCODING,
    //     HeaderValue::from_static("gzip, deflate, br, zstd"),
    // );
    headers.insert(
        header::ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7,en-DE;q=0.6,zh-TW;q=0.5"),
    );

    if !anonymous {
        headers.insert(header::HOST, HeaderValue::from_static("www.instagram.com"));
        headers.insert(header::ORIGIN, HeaderValue::from_static("https://www.instagram.com"));
        headers.insert("X-Instagram-AJAX", HeaderValue::from_static("1"));
        headers.insert("X-Requested-With", HeaderValue::from_static("XMLHttpRequest"));
    }

    headers.insert(header::REFERER, HeaderValue::from_static("https://www.instagram.com/"));

    // Browser identification
    headers.insert("Sec-Ch-Prefers-Color-Scheme", HeaderValue::from_static("dark"));
    headers.insert(
        "Sec-Ch-Ua",
        HeaderValue::from_static("\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\""),
    );
    headers.insert(
        "Sec-Ch-Ua-Full-Version-List",
        HeaderValue::from_static(
            "\"Google Chrome\";v=\"131.0.6778.266\", \"Chromium\";v=\"131.0.6778.266\", \"Not_A Brand\";v=\"24.0.0.0\"",
        ),
    );
    headers.insert("Sec-Ch-Ua-Mobile", HeaderValue::from_static("?0"));
    headers.insert("Sec-Ch-Ua-Model", HeaderValue::from_static(""));
    headers.insert("Sec-Ch-Ua-Platform", HeaderValue::from_static("\"macOS\""));
    headers.insert("Sec-Ch-Ua-Platform-Version", HeaderValue::from_static("\"14.3.0\""));

    // Security headers
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("empty"));
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));
    headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-origin"));

    // Priority and DNT
    // headers.insert("Priority", HeaderValue::from_static("u=1,i"));
    // headers.insert("Dnt", HeaderValue::from_static("1"));

    headers
}
