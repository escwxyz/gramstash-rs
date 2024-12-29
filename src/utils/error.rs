// use teloxide::RequestError;

// #[derive(Debug, thiserror::Error)]
// pub enum BotError {
//     #[error("Invalid URL: {0}")]
//     InvalidUrl(String),
//     #[error("Network error: {0}")]
//     NetworkError(String),
//     #[error("API error: {0}")]
//     ApiError(String),
//     #[error("Parse error: {0}")]
//     ParseError(String),
//     #[error("Redis error: {0}")]
//     RedisError(String),
//     #[error("Unsupported media: {0}")]
//     UnsupportedMedia(String),
//     #[error("Bot error: {0}")]
//     BotError(#[from] RequestError),
// }

// impl From<url::ParseError> for BotError {
//     fn from(err: url::ParseError) -> Self {
//         BotError::InvalidUrl(err.to_string())
//     }
// }

// impl From<redis::RedisError> for BotError {
//     fn from(err: redis::RedisError) -> Self {
//         BotError::RedisError(err.to_string())
//     }
// }

// impl From<BotError> for RequestError {
//     fn from(err: BotError) -> Self {
//         match err {
//             BotError::BotError(request_err) => request_err,
//             _ => RequestError::Api(teloxide::ApiError::Unknown(err.to_string())),
//         }
//     }
// }

// pub trait IntoResponseError<T> {
//     fn into_response_error(self) -> Result<T, teloxide::RequestError>;
// }

// impl<T> IntoResponseError<T> for Result<T, anyhow::Error> {
//     fn into_response_error(self) -> Result<T, teloxide::RequestError> {
//         self.map_err(|e| {
//             error!("Operation failed: {}", e);
//             teloxide::RequestError::Network(reqwest::Error::new())
//         })
//     }
// }
