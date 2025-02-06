#[derive(Debug, thiserror::Error)]
pub enum AuthenticationError {
    #[error("Login failed: Bad credentials")]
    BadCredentials,
    #[error("Two-factor authentication required")]
    TwoFactorRequired,
    #[error("Checkpoint verification required: {0}")]
    CheckpointRequired(String),
    #[error("Login failed: {0}")]
    LoginFailed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum InstagramError {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Auth error: {0}")]
    AuthenticationError(#[from] AuthenticationError),
    #[error("Invalid username: {0}")]
    InvalidUsername(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}
