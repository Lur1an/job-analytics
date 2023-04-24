use mongodb::bson::oid::ObjectId;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct ScrapedJob {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    job: job_scraper::Job,
}

pub struct Job {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    /// a hash generated with site-specific data to help recognize duplicates
    job_details: ai_analyzer::types::JobDetails,
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
