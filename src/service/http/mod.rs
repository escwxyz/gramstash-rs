use async_trait::async_trait;
use axum::http::HeaderValue;
use reqwest::{
    header::{self, HeaderMap},
    Client, Response,
};
use serde_json::Value;
use std::{sync::Arc, time::Duration};

use crate::platform::Platform;

#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn get(&self, url: &str) -> Result<Response, reqwest::Error>;
    async fn post(&self, url: &str, data: Option<Value>) -> Result<Response, reqwest::Error>;
    async fn get_json(&self, url: &str, params: Option<Value>) -> Result<Value, reqwest::Error>;
    #[allow(dead_code)]
    async fn post_json(&self, url: &str, data: Option<Value>) -> Result<Value, reqwest::Error>;
    async fn post_form(&self, url: &str, data: Option<Value>) -> Result<Value, reqwest::Error>;
    fn get_cookie_jar(&self) -> Arc<reqwest::cookie::Jar>;
    fn with_headers(&self, headers: HeaderMap) -> Box<dyn HttpClient>;
}

#[derive(Clone)]
pub struct HttpService {
    client: Client,
    cookie_jar: Arc<reqwest::cookie::Jar>,
    platform: Platform,
}

impl HttpService {
    pub fn new(platform: Platform) -> Result<Self, reqwest::Error> {
        let cookie_jar = Arc::new(reqwest::cookie::Jar::default());
        let client = Self::create_client(Arc::clone(&cookie_jar), Self::get_platform_headers(&platform))?;

        Ok(Self {
            client,
            cookie_jar,
            platform,
        })
    }

    fn get_platform_headers(platform: &Platform) -> HeaderMap {
        match platform {
            Platform::Instagram => {
                let mut headers = HeaderMap::new();
                headers.insert(header::HOST, HeaderValue::from_static("www.instagram.com"));
                headers.insert(header::ORIGIN, HeaderValue::from_static("https://www.instagram.com"));
                headers.insert("X-Instagram-AJAX", HeaderValue::from_static("1"));
                headers.insert("X-Requested-With", HeaderValue::from_static("XMLHttpRequest"));
                headers
            }
            _ => todo!(),
        }
    }

    fn create_client(cookie_jar: Arc<reqwest::cookie::Jar>, headers: HeaderMap) -> Result<Client, reqwest::Error> {
        let builder = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(30))
            .cookie_provider(cookie_jar)
            .default_headers(headers);

        #[cfg(debug_assertions)]
        let builder = builder.proxy(reqwest::Proxy::all("socks5://127.0.0.1:1080")?);

        builder.build()
    }

    // pub async fn get_json(
    //     &self,
    //     path: &str,
    //     params: &Value,
    //     host: Option<&str>,
    //     headers: Option<HeaderMap>,
    //     use_post: bool,
    // ) -> BotResult<Value> {
    //     let host = host.unwrap_or("www.instagram.com");
    //     let url = format!("https://{}/{}", host, path);

    //     // Determine query type for rate limiting (if implemented)
    //     let is_graphql_query = params.get("query_hash").is_some() && path.contains("graphql/query");
    //     let is_doc_id_query = params.get("doc_id").is_some() && path.contains("graphql/query");
    //     let is_iphone_query = host == "i.instagram.com";
    //     let is_other_query = !is_graphql_query && !is_doc_id_query && host == "www.instagram.com";

    //     info!(
    //         "Making {} request to {} (GraphQL: {}, DocID: {}, iPhone: {}, Other: {})",
    //         if use_post { "POST" } else { "GET" },
    //         url,
    //         is_graphql_query,
    //         is_doc_id_query,
    //         is_iphone_query,
    //         is_other_query
    //     );

    //     let response = if use_post {
    //         self.client
    //             .post(&url)
    //             .json(params)
    //             .headers(headers.unwrap_or_default())
    //             .send()
    //             .await
    //     } else {
    //         self.client
    //             .get(&url)
    //             .query(&params)
    //             .headers(headers.unwrap_or_default())
    //             .send()
    //             .await
    //     };

    //     self.handle_response(response).await
    // }

    // async fn handle_response(&self, response: Result<Response, reqwest::Error>) -> BotResult<Value> {
    //     match response {
    //         Ok(resp) => {
    //             // Handle redirects
    //             if resp.status().is_redirection() {
    //                 let redirect_url = resp
    //                     .headers()
    //                     .get("location")
    //                     .and_then(|h| h.to_str().ok())
    //                     .unwrap_or("");

    //                 info!("Redirected to: {}", redirect_url);

    //                 // Check for login redirects
    //                 if redirect_url.contains("/accounts/login") {
    //                     if redirect_url.contains("i.instagram.com") || redirect_url.contains("www.instagram.com") {
    //                         return Err(BotError::ServiceError(ServiceError::InstagramError(
    //                             InstagramError::LoginRequired,
    //                         )));
    //                     }
    //                 }
    //             }

    //             // Handle different status codes
    //             match resp.status() {
    //                 StatusCode::OK => {
    //                     let response_text = resp.text().await.map_err(|e| {
    //                         BotError::ServiceError(ServiceError::InstagramError(InstagramError::NetworkError(e)))
    //                     })?;

    //                     if response_text.trim().is_empty() {
    //                         return Err(BotError::ServiceError(ServiceError::InstagramError(
    //                             InstagramError::DeserializationError("Empty response body".into()),
    //                         )));
    //                     }

    //                     let data: Value = serde_json::from_str(&response_text).map_err(|e| {
    //                         BotError::ServiceError(ServiceError::InstagramError(InstagramError::DeserializationError(
    //                             format!("Failed to parse JSON: {}, Response: {}", e, response_text),
    //                         )))
    //                     })?;

    //                     // Check response status
    //                     if let Some(status) = data.get("status").and_then(|s| s.as_str()) {
    //                         if status != "ok" {
    //                             return Err(BotError::ServiceError(ServiceError::InstagramError(
    //                                 InstagramError::ApiError(format!("Instagram API returned status: {}", status)),
    //                             )));
    //                         }
    //                     }

    //                     Ok(data)
    //                 }
    //                 StatusCode::BAD_REQUEST => Err(BotError::ServiceError(ServiceError::InstagramError(
    //                     InstagramError::BadRequest(resp.text().await.unwrap_or_default()),
    //                 ))),
    //                 StatusCode::NOT_FOUND => Err(BotError::ServiceError(ServiceError::InstagramError(
    //                     InstagramError::NotFound(resp.text().await.unwrap_or_default()),
    //                 ))),
    //                 StatusCode::TOO_MANY_REQUESTS => Err(BotError::ServiceError(ServiceError::InstagramError(
    //                     InstagramError::TooManyRequests,
    //                 ))),
    //                 _ => Err(BotError::ServiceError(ServiceError::InstagramError(
    //                     InstagramError::NetworkError(resp.error_for_status().unwrap_err()),
    //                 ))),
    //             }
    //         }
    //         Err(e) => {
    //             info!("Request failed: {}", e);
    //             Err(BotError::ServiceError(ServiceError::InstagramError(
    //                 InstagramError::NetworkError(e),
    //             )))
    //         }
    //     }
    // }
}

#[async_trait]
impl HttpClient for HttpService {
    async fn get(&self, url: &str) -> Result<Response, reqwest::Error> {
        self.client.get(url).send().await.map_err(|e| e.into())
    }

    async fn post(&self, url: &str, data: Option<Value>) -> Result<Response, reqwest::Error> {
        let mut builder = self.client.post(url);
        if let Some(data) = data {
            builder = builder.json(&data);
        }
        builder.send().await
    }

    async fn get_json(&self, url: &str, params: Option<Value>) -> Result<Value, reqwest::Error> {
        let mut builder = self.client.get(url);
        if let Some(params) = params {
            builder = builder.query(&params);
        }
        let response = builder.send().await?;
        response.json().await
    }

    async fn post_json(&self, url: &str, data: Option<Value>) -> Result<Value, reqwest::Error> {
        let response = self.post(url, data).await?;
        response.json().await
    }

    async fn post_form(&self, url: &str, data: Option<Value>) -> Result<Value, reqwest::Error> {
        let mut builder = self.client.post(url);
        if let Some(data) = data {
            builder = builder.form(&data);
        }
        let response = builder.send().await?;
        response.json().await
    }

    fn get_cookie_jar(&self) -> Arc<reqwest::cookie::Jar> {
        Arc::clone(&self.cookie_jar)
    }

    fn with_headers(&self, headers: HeaderMap) -> Box<dyn HttpClient> {
        let client = Self::create_client(Arc::clone(&self.cookie_jar), headers).unwrap();

        Box::new(Self {
            client,
            cookie_jar: Arc::clone(&self.cookie_jar),
            platform: self.platform.clone(),
        })
    }
}
