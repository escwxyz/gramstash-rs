#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Authentication required")]
    AuthenticationRequired,
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Login failed: Bad credentials")]
    BadCredentials,
    #[error("Two-factor authentication required")]
    TwoFactorRequired,
    #[error("Checkpoint verification required: {0}")]
    CheckpointRequired(String),
    #[error("Cookie not found")]
    CookieNotFound,
    #[error("Logout failed: {0}")]
    LogoutFailed(String),
    #[error("Other error: {0}")]
    Other(String),
}

impl From<reqwest::Error> for AuthError {
    fn from(error: reqwest::Error) -> Self {
        AuthError::Other(error.to_string())
    }
}
