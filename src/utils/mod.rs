use anyhow::Result;
use url::Url;

pub mod error;
pub mod http;
pub mod redis;

pub fn parse_url(url: &str) -> Result<Url> {
    Url::parse(url).map_err(|_| anyhow::anyhow!("Invalid Instagram URL"))
}
