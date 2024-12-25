pub mod start;
pub mod help;
pub mod download;

use anyhow::Result;

pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
