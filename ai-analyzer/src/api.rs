use std::error::Error;
use thiserror::Error;
use async_trait::async_trait;


/// Trait for extracting structured data from raw text data
#[async_trait]
pub trait DataExtractor<T> {
    type E: Error + Send + Sync;
    async fn extract(&self, text: &str) -> Result<T, Self::E>;
}
