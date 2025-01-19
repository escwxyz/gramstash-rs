use rand::Rng;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client, Response, StatusCode,
};
use serde_json::Value;
use std::time::Duration;

use crate::error::{BotError, BotResult, InstagramError, ServiceError};

pub const INSTAGRAM_USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

pub const INSTAGRAM_IPHONE_USER_AGENT: &str =
    "Instagram 273.0.0.16.70 (iPad13,8; iOS 16_3; en_US; en-US; scale=2.00; 2048x2732; 452417278) AppleWebKit/420+";

#[derive(Clone, Debug)]
pub enum DeviceType {
    Desktop,
    IPhone,
}

#[derive(Clone)]
pub struct HttpService {
    client: Client,
}

impl HttpService {
    // TODO: improve
    pub fn new(anonymous: bool, device_type: DeviceType, client: Option<Client>) -> BotResult<Self> {
        info!("Initializing HttpService...");

        let client = if let Some(client) = client {
            client
        } else {
            let builder = Client::builder()
                .timeout(Duration::from_secs(30))
                .connect_timeout(Duration::from_secs(30))
                .default_headers(match device_type {
                    DeviceType::Desktop => build_desktop_instagram_headers(anonymous),
                    DeviceType::IPhone => build_iphone_instagram_headers(),
                })
                .user_agent(match device_type {
                    DeviceType::Desktop => INSTAGRAM_USER_AGENT,
                    DeviceType::IPhone => INSTAGRAM_IPHONE_USER_AGENT,
                });

            build_client(builder)?
        };

        info!("HttpService initialized");
        Ok(Self { client })
    }

    // pub async fn get_iphone_json(&self, path: &str, params: &Value, session: &SessionData) -> BotResult<Value> {
    //     // Create a new client with iPhone headers
    //     let mut headers = build_iphone_instagram_headers();

    //     // Add user-specific headers
    //     headers.insert("ig-intended-user-id", HeaderValue::from_str(&session.user_id).unwrap());

    //     // Add timestamp header
    //     let timestamp = chrono::Utc::now().timestamp_millis() as f64 / 1000.0;
    //     headers.insert(
    //         "x-pigeon-rawclienttime",
    //         HeaderValue::from_str(&format!("{:.6}", timestamp)).unwrap(),
    //     );

    //     // Map session data to iPhone headers
    //     let header_mappings = [
    //         ("x-mid", &session.mid),
    //         ("ig-u-ds-user-id", &session.ds_user_id),
    //         ("x-ig-device-id", &session.ig_did),
    //         ("x-ig-family-device-id", &session.ig_did),
    //         ("family_device_id", &session.ig_did),
    //     ];

    //     // Add headers from session data
    //     for (header_name, cookie) in header_mappings.iter() {
    //         headers.insert(*header_name, HeaderValue::from_str(&cookie.value).unwrap());
    //     }

    //     // Remove desktop-specific headers
    //     let desktop_headers = ["Host", "Origin", "X-Instagram-AJAX", "X-Requested-With", "Referer"];
    //     for header in desktop_headers.iter() {
    //         headers.remove(*header);
    //     }

    //     // Make the request
    //     let response = self
    //         .get_json(path, params, Some("i.instagram.com"), Some(headers), false)
    //         .await?;

    //     info!("Response: {}", response);

    //     Ok(response)
    // }

    pub async fn get_json(
        &self,
        path: &str,
        params: &Value,
        host: Option<&str>,
        headers: Option<HeaderMap>,
        use_post: bool,
    ) -> BotResult<Value> {
        let host = host.unwrap_or("www.instagram.com");
        let url = format!("https://{}/{}", host, path);

        // Determine query type for rate limiting (if implemented)
        let is_graphql_query = params.get("query_hash").is_some() && path.contains("graphql/query");
        let is_doc_id_query = params.get("doc_id").is_some() && path.contains("graphql/query");
        let is_iphone_query = host == "i.instagram.com";
        let is_other_query = !is_graphql_query && !is_doc_id_query && host == "www.instagram.com";

        info!(
            "Making {} request to {} (GraphQL: {}, DocID: {}, iPhone: {}, Other: {})",
            if use_post { "POST" } else { "GET" },
            url,
            is_graphql_query,
            is_doc_id_query,
            is_iphone_query,
            is_other_query
        );

        let response = if use_post {
            self.client
                .post(&url)
                .json(params)
                .headers(headers.unwrap_or_default())
                .send()
                .await
        } else {
            self.client
                .get(&url)
                .query(&params)
                .headers(headers.unwrap_or_default())
                .send()
                .await
        };

        self.handle_response(response).await
    }

    async fn handle_response(&self, response: Result<Response, reqwest::Error>) -> BotResult<Value> {
        match response {
            Ok(resp) => {
                // Handle redirects
                if resp.status().is_redirection() {
                    let redirect_url = resp
                        .headers()
                        .get("location")
                        .and_then(|h| h.to_str().ok())
                        .unwrap_or("");

                    info!("Redirected to: {}", redirect_url);

                    // Check for login redirects
                    if redirect_url.contains("/accounts/login") {
                        if redirect_url.contains("i.instagram.com") || redirect_url.contains("www.instagram.com") {
                            return Err(BotError::ServiceError(ServiceError::InstagramError(
                                InstagramError::LoginRequired,
                            )));
                        }
                    }
                }

                // Handle different status codes
                match resp.status() {
                    StatusCode::OK => {
                        let response_text = resp.text().await.map_err(|e| {
                            BotError::ServiceError(ServiceError::InstagramError(InstagramError::NetworkError(e)))
                        })?;

                        if response_text.trim().is_empty() {
                            return Err(BotError::ServiceError(ServiceError::InstagramError(
                                InstagramError::DeserializationError("Empty response body".into()),
                            )));
                        }

                        let data: Value = serde_json::from_str(&response_text).map_err(|e| {
                            BotError::ServiceError(ServiceError::InstagramError(InstagramError::DeserializationError(
                                format!("Failed to parse JSON: {}, Response: {}", e, response_text),
                            )))
                        })?;

                        // Check response status
                        if let Some(status) = data.get("status").and_then(|s| s.as_str()) {
                            if status != "ok" {
                                return Err(BotError::ServiceError(ServiceError::InstagramError(
                                    InstagramError::ApiError(format!("Instagram API returned status: {}", status)),
                                )));
                            }
                        }

                        Ok(data)
                    }
                    StatusCode::BAD_REQUEST => Err(BotError::ServiceError(ServiceError::InstagramError(
                        InstagramError::BadRequest(resp.text().await.unwrap_or_default()),
                    ))),
                    StatusCode::NOT_FOUND => Err(BotError::ServiceError(ServiceError::InstagramError(
                        InstagramError::NotFound(resp.text().await.unwrap_or_default()),
                    ))),
                    StatusCode::TOO_MANY_REQUESTS => Err(BotError::ServiceError(ServiceError::InstagramError(
                        InstagramError::TooManyRequests,
                    ))),
                    _ => Err(BotError::ServiceError(ServiceError::InstagramError(
                        InstagramError::NetworkError(resp.error_for_status().unwrap_err()),
                    ))),
                }
            }
            Err(e) => {
                info!("Request failed: {}", e);
                Err(BotError::ServiceError(ServiceError::InstagramError(
                    InstagramError::NetworkError(e),
                )))
            }
        }
    }
}

fn build_desktop_instagram_headers(anonymous: bool) -> HeaderMap {
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

    headers
}

fn build_iphone_instagram_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();

    // Basic headers
    headers.insert("x-ads-opt-out", HeaderValue::from_static("1"));
    headers.insert("x-bloks-is-panorama-enabled", HeaderValue::from_static("true"));
    headers.insert(
        "x-bloks-version-id",
        HeaderValue::from_static("01507c21540f73e2216b6f62a11a5b5e51aa85491b72475c080da35b1228ddd6"),
    );
    headers.insert("x-fb-client-ip", HeaderValue::from_static("True"));
    headers.insert("x-fb-connection-type", HeaderValue::from_static("wifi"));
    headers.insert("x-fb-http-engine", HeaderValue::from_static("Liger"));
    headers.insert("x-fb-server-cluster", HeaderValue::from_static("True"));
    headers.insert("x-fb", HeaderValue::from_static("1"));

    // Instagram specific headers
    headers.insert("x-ig-abr-connection-speed-kbps", HeaderValue::from_static("2"));
    headers.insert("x-ig-app-id", HeaderValue::from_static("124024574287414"));
    headers.insert("x-ig-app-locale", HeaderValue::from_static("en-US"));
    headers.insert("x-ig-app-startup-country", HeaderValue::from_static("US"));
    headers.insert("x-ig-bandwidth-speed-kbps", HeaderValue::from_static("0.000"));
    headers.insert("x-ig-capabilities", HeaderValue::from_static("36r/F/8="));
    headers.insert("x-ig-connection-type", HeaderValue::from_static("WiFi"));
    headers.insert("x-ig-device-locale", HeaderValue::from_static("en-US"));
    headers.insert("x-ig-mapped-locale", HeaderValue::from_static("en-US"));
    headers.insert("x-ig-www-claim", HeaderValue::from_static("0"));

    // Random/dynamic headers
    headers.insert(
        "x-ig-connection-speed",
        HeaderValue::from_str(&format!("{}kbps", rand::thread_rng().gen_range(1000..20000))).unwrap(),
    );
    headers.insert(
        "x-pigeon-session-id",
        HeaderValue::from_str(&uuid::Uuid::new_v4().to_string()).unwrap(),
    );
    headers.insert("x-tigon-is-retry", HeaderValue::from_static("False"));
    headers.insert("x-whatsapp", HeaderValue::from_static("0"));

    headers
}

fn build_client(builder: reqwest::ClientBuilder) -> BotResult<reqwest::Client> {
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
