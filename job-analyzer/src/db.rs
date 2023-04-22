use crate::{JobDetails, JobPost};
use lazy_static::lazy_static;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Job {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    job_details: JobDetails,
    job_post: JobPost,
}

trait Repository<T> {
    fn save(&self, item: T) -> Result<(), mongodb::error::Error>;
    fn find_by_id(&self, id: ObjectId) -> Result<T, mongodb::error::Error>;
    fn find_all(&self) -> Result<Vec<T>, mongodb::error::Error>;
    fn delete(&self, id: ObjectId) -> Result<(), mongodb::error::Error>;
}

pub async fn connect(mongodb_connection_url: &str, database_name: &str) -> mongodb::Database {
    let client = mongodb::Client::with_uri_str(mongodb_connection_url)
        .await
        .expect("Incorrect mongodb connection url");
    client.database(database_name)
}
