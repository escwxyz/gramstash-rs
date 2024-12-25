use teloxide::RequestError;

#[derive(Debug, thiserror::Error)]
pub enum BotError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Unsupported media: {0}")]
    UnsupportedMedia(String),
    #[error("Bot error: {0}")]
    BotError(#[from] RequestError),
}

// Implement conversion from url::ParseError
impl From<url::ParseError> for BotError {
    fn from(err: url::ParseError) -> Self {
        BotError::InvalidUrl(err.to_string())
    }
}