use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    hash::{Hash, Hasher},
};

use futures::{StreamExt, TryStreamExt};
use mongodb::bson::doc;
use tokio::sync::Mutex;

use crate::Target;

pub async fn fix(site: Target) {
    let mongodb_connection_url =
        std::env::var("MONGODB_CONNECTION_URL").expect("MONGODB_CONNECTION_URL not set");
    let database_name = std::env::var("DATABASE").expect("DATABASE not set");
    let db = persistence::connect(&mongodb_connection_url, &database_name).await;
    let collection = db.collection::<persistence::ScrapedJob>("scraped-jobs");
    match site {
        Target::Xing => todo!(),
        Target::Linkedin => {
            let filter = doc! {
                "job.type": "Linkedin"
            };
            log::info!("Filter for query: {}", filter);
            let jobs = collection
                .find(None, None)
                .await
                .expect("Query shouldn't fail")
                .collect::<Vec<_>>()
                .await;
            log::info!("Found {} linkedin jobs", jobs.len());
            let mut key_set = HashSet::new();
            let mut delete_after = Vec::new();
            for job in jobs.into_iter().filter_map(Result::ok) {
                let mut hasher = DefaultHasher::new();
                match job.job {
                    job_scraper::Job::Linkedin { job: linkedin_job } => {
                        if linkedin_job.title.is_none() || linkedin_job.company.name.is_none() {
                            delete_after.push(job.id);
                            continue;
                        }
                        let title = linkedin_job.title.unwrap();
                        title.replace(" ", "").hash(&mut hasher);
                        linkedin_job
                            .company
                            .name
                            .unwrap()
                            .replace(" ", "")
                            .hash(&mut hasher);
                        let key = hasher.finish();
                        if key_set.contains(&key) {
                            log::info!("Duplicated detected: {}", title);
                            delete_after.push(job.id);
                        } else {
                            key_set.insert(key);
                        }
                    }
                    _ => {
                        continue;
                    }
                }
            }
            log::info!("Deleting {} jobs", delete_after.len());
            let delete_result = collection
                .delete_many(doc! {"_id": {"$in": delete_after}}, None)
                .await;
            log::info!("Query result: {:?}", delete_result);
        }
        Target::Stepstone => todo!(),
        Target::Glassdoor => todo!(),
        Target::Instaffo => todo!(),
    }
}
