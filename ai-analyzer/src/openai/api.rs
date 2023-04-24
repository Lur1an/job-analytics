use thiserror::Error;

pub struct Client {
    client: reqwest::Client,
    api_key: String,
}

#[derive(Debug, Error)]
pub enum Error {}

pub type Result<T> = std::result::Result<T, Error>;
