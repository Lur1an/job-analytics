use ai_analyzer::types::JobDetails;
use async_trait::async_trait;
use mongodb::{bson::oid::ObjectId, results::InsertManyResult};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

#[derive(Serialize, Debug, Deserialize)]
pub struct ScrapedJob {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub job: job_scraper::Job,
    site_hash: String,
    pub analyzed: bool,
}

impl ScrapedJob {
    pub fn new(job: job_scraper::Job) -> Self {
        let mut state = DefaultHasher::new();
        job.hash(&mut state);
        Self {
            id: None,
            job,
            site_hash: state.finish().to_string(),
            analyzed: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Job {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    /// a hash generated with site-specific data to help recognize duplicates
    job_details: JobDetails,
    private: bool,
    title: String,
    link: Option<String>,
    site_hash: String,
}

pub const COLLECTION_JOBS: &str = "analyzed-jobs";

pub async fn connect(mongodb_connection_url: &str, database_name: &str) -> mongodb::Database {
    let client = mongodb::Client::with_uri_str(mongodb_connection_url)
        .await
        .expect("Incorrect mongodb connection url");
    client.database(database_name)
}

/// Saves multiple documents, ignoring duplicate key errors
pub async fn save_many<T>(
    col: &mongodb::Collection<T>,
    docs: impl Iterator<Item = T>,
) -> Result<InsertManyResult, mongodb::error::Error>
where
    T: Serialize,
{
    let options = mongodb::options::InsertManyOptions::builder()
        .ordered(false)
        .build();
    col.insert_many(docs, options).await
}

#[cfg(test)]
mod tests {
    use super::*;
}
