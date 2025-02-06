use crate::{config::ConfigError, error::BotError};

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("queue error: {0}")]
    QueueError(String),
    #[error("recv error: {0}")]
    RecvError(String),
    #[error("task error: {0}")]
    TaskError(String),
    #[error("other error: {0}")]
    Other(String),
}

impl From<BotError> for RuntimeError {
    fn from(error: BotError) -> Self {
        match error {
            BotError::RuntimeError(e) => e,
            _ => RuntimeError::Other(error.to_string()),
        }
    }
}

impl From<ConfigError> for RuntimeError {
    fn from(error: ConfigError) -> Self {
        RuntimeError::Other(error.to_string())
    }
}
