use ai_analyzer::openai_analyzer::{create_job, init};
use tokio::fs::File;
use ai_analyzer::types::JobPost;
use futures::{stream, StreamExt};
use serde_json::to_string;
use crate::Target;

pub async fn analyze(site: Target) {
    todo!()
    // let mongodb_connection_url =
    //     std::env::var("MONGODB_CONNECTION_URL").expect("MONGODB_CONNECTION_URL not set");
    // let database_name = std::env::var("DATABASE").expect("DATABASE not set");
    // let db = connect(&mongodb_connection_url, &database_name).await;
    // log::info!("Connected to database");
    //
    // let filename = site.filename();
    // log::info!("Analyzing {}", filename);
    // let file = File::open(filename)
    //     .await
    //     .expect("Failed to open json data file");
    // let bytes = file.metadata().await.expect("Failed to get metadata").len();
    // log::info!("File size: {} bytes", bytes);
    // let data = tokio::fs::read_to_string(filename)
    //     .await
    //     .expect("Failed to read file");
    // let data: Vec<JobPost> = serde_json::from_str(&data).expect("Failed to parse json");
    // stream::iter(data)
    //     .map(create_job)
    //     .buffer_unordered(100)
    //     .for_each(|job| {
    //         let db = db.clone();
    //         async move {
    //             let job_json = to_string(&job).expect("Failed to serialize job");
    //             log::debug!("Saving job: {}", job_json);
    //             let db_result = save_job(db, &job).await;
    //             match db_result {
    //                 Ok(rs) => log::debug!("Saved job, id: {:?}", rs),
    //                 Err(e) => log::error!("Failed to save job: {}", e),
    //             }
    //         }
    //     })
    //     .await;
    // log::info!("Finished analyzing & persisting {}", filename);
}
