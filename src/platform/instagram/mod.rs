mod error;
pub mod model;
mod util;

use std::{any::Any, collections::HashMap, time::Duration};

use anyhow::Context;
use async_trait::async_trait;
use axum::http::{HeaderMap, HeaderValue};
use chrono::Utc;
use model::{InstagramIdentifier, InstagramMedia, XDTGraphMedia};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use teloxide::{
    adaptors::Throttle,
    prelude::Requester,
    types::{ChatId, InputFile, InputMedia, InputMediaAudio, InputMediaPhoto, InputMediaVideo},
    Bot,
};
use url::Url;

pub use error::*;
pub use util::*;

use crate::{
    config::AppConfig,
    error::HandlerResult,
    service::{
        http::{HttpClient, HttpService},
        AuthData, AuthError, Credentials, PlatformAuth, PlatformSession, PlatformSessionData, Session, SessionData,
        SessionError, SessionStatus,
    },
    state::AppState,
};

use super::{model::PlatformIdentifier, MediaFile, Platform, PlatformCapability, PlatformError};

pub struct PlatformInstagram {
    http_service: HttpService,
}

#[async_trait]
impl PlatformCapability for PlatformInstagram {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn platform_id(&self) -> Platform {
        Platform::Instagram
    }

    fn platform_name(&self) -> &str {
        "Instagram"
    }

    async fn parse_url(&self, url_str: &str) -> Result<PlatformIdentifier, PlatformError> {
        let url = self.validate_url(url_str).await?;

        let path_segments: Vec<_> = url
            .path_segments()
            .ok_or_else(|| PlatformError::ParsingError("No path segments".into()))?
            .collect();

        match path_segments.as_slice() {
            ["stories", username, story_id] => Ok(PlatformIdentifier::Instagram(InstagramIdentifier::Story {
                username: username.to_string(),
                story_id: story_id.to_string(),
            })),

            ["p", post_id, ..] => Ok(PlatformIdentifier::Instagram(InstagramIdentifier::Post {
                shortcode: post_id.to_string(),
            })),

            ["reel", shortcode, ..] => Ok(PlatformIdentifier::Instagram(InstagramIdentifier::Reel {
                shortcode: shortcode.to_string(),
            })),

            _ => Err(PlatformError::ParsingError("Invalid URL format".into())),
        }
    }

    async fn fetch_resource(&self, identifier: &PlatformIdentifier) -> HandlerResult<MediaFile> {
        match identifier {
            PlatformIdentifier::Instagram(InstagramIdentifier::Story {
                username: _,
                story_id: _,
            }) => {
                // TODO
                todo!()
            }
            PlatformIdentifier::Instagram(
                InstagramIdentifier::Post { shortcode } | InstagramIdentifier::Reel { shortcode },
            ) => {
                let doc_id = "8845758582119845";

                let variables = serde_json::json!({
                    "shortcode": shortcode
                });

                let params = serde_json::json!({
                    "doc_id": doc_id,
                    "variables": variables.to_string(),
                    "server_timestamps": "true",
                });

                let response = self
                    .http_service
                    .get_json("https://www.instagram.com/graphql/query/", Some(params))
                    .await?;

                // TODO: sometimes response is null when status is ok
                // {"data": Object {"xdt_shortcode_media": Null}, "extensions": Object {"is_final": Bool(true)}, "status": String("ok")}

                let media_value = response
                    .get("data")
                    .and_then(|d| d.get("xdt_shortcode_media"))
                    .ok_or_else(|| PlatformError::ParsingError("Missing xdt_shortcode_media".to_string()))?;

                let media_data = serde_json::from_value::<XDTGraphMedia>(media_value.clone())
                    .map_err(|e| PlatformError::ParsingError(format!("Failed to deserialize media: {}", e)))?;

                let media = match media_data {
                    XDTGraphMedia::Image(image) => TryInto::<InstagramMedia>::try_into(image)?,
                    XDTGraphMedia::Video(video) => TryInto::<InstagramMedia>::try_into(video)?,
                    XDTGraphMedia::Sidecar(sidecar) => TryInto::<InstagramMedia>::try_into(sidecar)?,
                };

                return Ok(media.try_into()?);
            }
        }
    }
    #[allow(unused)]
    async fn pre_process(
        &self,
        bot: &Throttle<Bot>,
        chat_id: ChatId,
        media_file: &MediaFile,
    ) -> HandlerResult<MediaFile> {
        // TODO
        Ok(media_file.clone())
    }
    #[allow(unused)]
    async fn post_process(&self, bot: &Throttle<Bot>, chat_id: ChatId, media_file: &MediaFile) -> HandlerResult<()> {
        // TODO
        Ok(())
    }

    async fn send_to_telegram(
        &self,
        bot: &Throttle<Bot>,
        chat_id: ChatId,
        media_file: &MediaFile,
    ) -> HandlerResult<()> {
        let cache_service = AppState::get()?.service_registry.cache;

        let secs = AppConfig::get()?.service.cache.ttl;

        let ttl = Duration::from_secs(secs);

        cache_service.set::<MediaFile>(media_file.clone(), ttl).await?;

        if media_file.items.len() == 1 {
            let item = media_file.items.first().unwrap();

            let _ = match item.media_type {
                super::MediaType::Image => bot.send_photo(chat_id, InputFile::url(item.url.clone())).await?,
                super::MediaType::Video => bot.send_video(chat_id, InputFile::url(item.url.clone())).await?,
                super::MediaType::Audio => bot.send_audio(chat_id, InputFile::url(item.url.clone())).await?,
            };
        } else {
            let media_group = media_file
                .items
                .iter()
                .map(|item| match item.media_type {
                    super::MediaType::Image => {
                        InputMedia::Photo(InputMediaPhoto::new(InputFile::url(item.url.clone())))
                    }
                    super::MediaType::Video => {
                        InputMedia::Video(InputMediaVideo::new(InputFile::url(item.url.clone())))
                    }
                    super::MediaType::Audio => {
                        InputMedia::Audio(InputMediaAudio::new(InputFile::url(item.url.clone())))
                    }
                })
                .collect::<Vec<_>>();

            bot.send_media_group(chat_id, media_group).await?;
        }

        Ok(())
    }
}

impl PlatformInstagram {
    pub fn new() -> Result<Self, InstagramError> {
        let http_service = HttpService::new(Platform::Instagram)?;
        Ok(Self { http_service })
    }

    const REQUIRED_COOKIES: [(&'static str, Duration); 5] = [
        ("sessionid", Duration::from_secs(365 * 24 * 60 * 60)), // Primary auth token
        ("ds_user_id", Duration::from_secs(365 * 24 * 60 * 60)), // User identifier
        ("csrftoken", Duration::from_secs(365 * 24 * 60 * 60)), // Required for POST requests
        ("ig_did", Duration::from_secs(365 * 24 * 60 * 60)),    // Device ID
        ("mid", Duration::from_secs(365 * 24 * 60 * 60)),       // Machine ID
    ];

    async fn validate_url(&self, url: &str) -> Result<Url, PlatformError> {
        let parsed_url =
            Url::parse(url).map_err(|_| PlatformError::Instagram(InstagramError::InvalidUrl(url.to_string())))?;

        if parsed_url.host_str() == Some("instagram.com") || parsed_url.host_str() == Some("www.instagram.com") {
            Ok(parsed_url)
        } else {
            Err(PlatformError::Instagram(InstagramError::InvalidUrl(
                "Not an Instagram URL".into(),
            )))
        }
    }

    async fn perform_login(&self, username: &str, password: &str, csrf_token: &str) -> Result<Value, InstagramError> {
        let enc_password = format!("#PWD_INSTAGRAM_BROWSER:0:{}:{}", Utc::now().timestamp(), password);

        let mut headers = HeaderMap::new();
        headers.insert("X-CSRFToken", csrf_token.parse().unwrap());
        headers.insert(
            "Content-Type",
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );

        let http_service = self.http_service.with_headers(headers);

        let response = http_service
            .post_form(
                "https://www.instagram.com/api/v1/web/accounts/login/ajax/",
                Some(serde_json::json!(
                    {
                        "username": username,
                        "enc_password": enc_password,
                        "queryParams": "{}",
                        "optIntoOneTap": "false",
                        "trustedDeviceRecords": "{}",
                    }
                )),
            )
            .await
            .unwrap();

        Ok(response)
    }

    async fn generate_session_data(&self, user_id: &str, username: &str) -> Result<SessionData, AuthError> {
        let mut cookies = HashMap::new();

        for (name, duration) in Self::REQUIRED_COOKIES {
            let cookie = self.extract_cookie(name, duration)?;
            cookies.insert(name.to_string(), cookie);
        }

        Ok(SessionData {
            auth_data: AuthData {
                cookies,
                tokens: HashMap::new(),
            },
            platform_data: PlatformSessionData::Instagram(InstagramSessionData {
                user_id: user_id.to_string(),
                username: username.to_string(),
                authenticated: true,
            }),
        })
    }

    // Fetch user IDs from the stories feed
    // async fn fetch_user_ids(
    //     &self,
    //     http: &HttpService,
    //     target_instagram_username: Option<&str>,
    // ) -> Result<Vec<String>, PlatformError> {
    //     info!("Fetching user IDs...");
    //     let variables = serde_json::json!({
    //         "only_stories": true
    //     });

    //     let params = serde_json::json!({
    //         "query_hash": "d15efd8c0c5b23f0ef71f18bf363c704",
    //         "variables": variables.to_string(),
    //     });

    //     let stories_data = http.get_json("graphql/query", &params, None, None, true).await.unwrap();

    //     let edges = stories_data["data"]["user"]["feed_reels_tray"]["edge_reels_tray_to_reel"]["edges"]
    //         .as_array()
    //         .ok_or_else(|| PlatformError::ParsingError("Failed to get edges from response".to_string()))?;

    //     // {
    //     //     "node": {
    //     //       "can_reply": true,
    //     //       "expiring_at": 1737431150,
    //     //       "id": "2267629874",
    //     //       "latest_reel_media": 1737295160,
    //     //       "muted": false,
    //     //       "prefetch_count": 0,
    //     //       "ranked_position": 2,
    //     //       "seen": null,
    //     //       "seen_ranked_position": 2,
    //     //       "user": {
    //     //         "id": "2267629874",
    //     //         "profile_pic_url": "https://scontent-lax3-2.cdninstagram.com/v/t51.2885-19/462178502_1075515874120935_2177629506287086586_n.jpg?stp=dst-jpg_s150x150_tt6&_nc_ht=scontent-lax3-2.cdninstagram.com&_nc_cat=103&_nc_ohc=jT9Z65UUNlQQ7kNvgEFuG-i&_nc_gid=3bf0da0658334e07baa3647812cf6c9e&edm=APrQDZQBAAAA&ccb=7-5&oh=00_AYBJQoqI8fWIsVpysHZ8v6VSQfl66ebI8QVu1zG1DuZIQQ&oe=67937F46&_nc_sid=01a934",
    //     //         "username": "yvweii_"
    //     //       }
    //     //     }
    //     //   },

    //     // Use iterator chaining for cleaner collection
    //     let user_ids: Vec<String> = edges
    //         .iter()
    //         .filter_map(|edge| {
    //             let id = edge["node"]["id"].as_str()?;
    //             let username = edge["node"]["user"]["username"].as_str()?;

    //             match target_instagram_username {
    //                 Some(target) if username == target => Some(id.to_string()),
    //                 None => Some(id.to_string()),
    //                 _ => None,
    //             }
    //         })
    //         .collect();

    //     Ok(user_ids)
    // }
}

#[async_trait]
impl PlatformAuth for PlatformInstagram {
    fn get_http_service(&self) -> HttpService {
        self.http_service.clone()
    }

    async fn login(&self, credentials: &Credentials) -> Result<SessionData, AuthError> {
        self.http_service
            .get("https://www.instagram.com/")
            .await
            .context("Failed to get Instagram homepage")
            .unwrap();
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let csrf_token = self.get_csrf_token().await.context("Failed to get CSRF token").unwrap();

        let response = self
            .perform_login(&credentials.indentifier, &credentials.password, &csrf_token)
            .await
            .context("Failed to perform login")
            .unwrap();

        let user_id = response
            .get("user_id")
            .context("Failed to get user ID")
            .unwrap()
            .as_str()
            .context("User ID is not a string")
            .unwrap();
        let username = credentials.indentifier.clone();

        let session_data = self.generate_session_data(user_id, username.as_str()).await?;

        Ok(session_data)
    }

    async fn verify_session(&self, session: &Session) -> Result<bool, AuthError> {
        if let Some(session_data) = &session.session_data {
            for (name, _) in Self::REQUIRED_COOKIES {
                if !session_data.auth_data.cookies.contains_key(name) {
                    return Ok(false);
                }
            }

            let response = self
                .http_service
                .get("https://www.instagram.com/accounts/edit/")
                .await?;

            Ok(response.status().is_success())
        } else {
            Ok(false)
        }
    }
}

#[async_trait]
impl PlatformSession for PlatformInstagram {
    async fn validate_session(&self, session: &Session) -> Result<SessionStatus, SessionError> {
        let state = AppState::get().unwrap();
        let auth_service = state.service_registry.auth.lock().await;
        auth_service
            .verify_session(session)
            .await
            .map(|valid| {
                if valid {
                    SessionStatus::Active
                } else {
                    SessionStatus::Invalid
                }
            })
            .map_err(|_| SessionError::SessionInvalid)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstagramSessionData {
    pub user_id: String,
    pub username: String,
    pub authenticated: bool,
}
