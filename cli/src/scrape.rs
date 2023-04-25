use crate::Target;
use futures::StreamExt;
use persistence::save_many;
use tokio::fs::File;

pub async fn scrape(site: Target) {
    let mongodb_connection_url =
        std::env::var("MONGODB_CONNECTION_URL").expect("MONGODB_CONNECTION_URL not set");
    let database_name = std::env::var("DATABASE").expect("DATABASE not set");
    let db = persistence::connect(&mongodb_connection_url, &database_name).await;
    let collection = db.collection::<persistence::ScrapedJob>("scraped-jobs");
    match site {
        Target::Xing => {
            let queries = DEFAULT_SEARCH_QUERIES
                .into_iter()
                .map(String::from)
                .collect();
            let results = job_scraper::xing::scraper::scrape_queries(queries)
                .await
                .buffer_unordered(500)
                .chunks(500);
            tokio::pin!(results);
            while let Some(result_chunk) = results.next().await {
                let scraped_jobs = result_chunk
                    .into_iter()
                    .map(|job| persistence::ScrapedJob::new(job));
                match save_many(&collection, scraped_jobs).await {
                    Ok(insert_result) => {
                        log::info!("Inserted {} jobs", insert_result.inserted_ids.len())
                    }
                    Err(e) => log::error!("Error inserting scraped jobs: {}", e),
                }
            }
        }
        Target::Instaffo => {
            println!("Please enter your session cookie:");
            let mut session_cookie = String::new();
            std::io::stdin()
                .read_line(&mut session_cookie)
                .expect("Failed to read line");
            let session_cookie = session_cookie.trim();
            let results = job_scraper::instaffo::scrape(session_cookie.into())
                .await
                .chunks(20);
            tokio::pin!(results);
            while let Some(result_chunk) = results.next().await {
                let scraped_jobs = result_chunk
                    .into_iter()
                    .map(|job| persistence::ScrapedJob::new(job));
                match save_many(&collection, scraped_jobs).await {
                    Ok(insert_result) => {
                        log::info!("Inserted {} jobs", insert_result.inserted_ids.len())
                    }
                    Err(e) => log::error!("Error inserting scraped jobs: {}", e),
                }
            }
        }
        _ => {}
    }
}

const DEFAULT_SEARCH_QUERIES: [&str; 27] = [
    "Swift",
    "React native",
    "Flutter",
    "Software Engineer",
    "Cloud",
    "Devops",
    "Kubernetes",
    "Java EE",
    "Go Programming Language",
    "Elixir",
    "Kotlin",
    "C%2B%2B",
    "C%23",
    "Dotnet",
    "Spring Boot",
    "Microservices",
    "Python Developer",
    "Linux Software",
    "Linux",
    "Software Engineer",
    "Backend Software Engineer",
    "Fullstack Software Engineer",
    "Rust Software Engineer",
    "Solid JS",
    "Svelte",
    "NextJS",
    "Python",
];
