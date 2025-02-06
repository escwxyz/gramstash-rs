use super::instagram::InstagramError;

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("resource error: {0}")]
    ResourceError(String),
    #[error("parsing error: {0}")]
    ParsingError(String),
    #[error("Instagram error: {0}")]
    Instagram(#[from] InstagramError),
    #[error("Invalid platform: {0}")]
    InvalidPlatform(String),
}
