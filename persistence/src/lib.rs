use ai_analyzer::types::JobDetails;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ScrapedJob {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    job: job_scraper::Job,
}

impl ScrapedJob {
    pub fn new(job: job_scraper::Job) -> Self {
        Self { id: None, job }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Job {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    /// a hash generated with site-specific data to help recognize duplicates
    job_details: JobDetails,
    title: String,
    link: String,
    site_hash: u64,
}

pub async fn connect(mongodb_connection_url: &str, database_name: &str) -> mongodb::Database {
    let client = mongodb::Client::with_uri_str(mongodb_connection_url)
        .await
        .expect("Incorrect mongodb connection url");
    client.database(database_name)
}

#[cfg(test)]
mod tests {
    use super::*;
}
