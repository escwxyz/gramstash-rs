#[derive(thiserror::Error, Debug)]
pub enum SessionError {
    #[error("Session not found")]
    SessionNotFound,
    #[error("Session is stale")]
    SessionStale,
    #[error("Session is invalid")]
    SessionInvalid,
    #[error("Cache error: {0}")]
    CacheError(String),
}
