pub mod scraper;
pub(crate) mod types;

use thiserror::Error;

type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, Error)]
pub enum Error {
    #[error("Request error: '{0}'")]
    Request(#[from] reqwest::Error),
    #[error("Failed to scrape data from: '{0}'")]
    RequestNotOk(String),
    #[error("File error: '{0}'")]
    IoError(#[from] std::io::Error),
    #[error("Content not found in html: '{0}'")]
    ContentNotFound(&'static str),
}
