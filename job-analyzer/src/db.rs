use crate::{JobDetails, JobPost};
use mongodb::{bson::oid::ObjectId, results::InsertManyResult};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Job {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    job_details: JobDetails,
    job_post: JobPost,
}

pub async fn connect(mongodb_connection_url: &str, database_name: &str) -> mongodb::Database {
    let client = mongodb::Client::with_uri_str(mongodb_connection_url)
        .await
        .expect("Incorrect mongodb connection url");
    client.database(database_name)
}

pub async fn save_all_jobs(
    db: &mongodb::Database,
    jobs: &Vec<Job>,
) -> Result<InsertManyResult, mongodb::error::Error> {
    let collection = db.collection::<Job>("jobs");
    collection.insert_many(jobs, None).await
}
