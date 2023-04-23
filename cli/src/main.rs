use clap::{Parser, Subcommand};
use dotenv::dotenv;
use futures::{stream, StreamExt};
use job_analyzer::{
    db::{connect, save_job},
    openai_analyzer::{create_job, init},
    JobPost, Site,
};
use job_scraper::xing::scrape_queries;
use serde_json::to_string;
use tokio::fs::File;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// List of job-providers to execute the command against
    #[clap(long)]
    site: Vec<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Scrape {},
    Analyze {},
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

async fn scrape(site: Site) {
    match site {
        Site::Xing => {
            let queries = DEFAULT_SEARCH_QUERIES
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>();
            let file = File::create(site.filename())
                .await
                .expect("Failed to create file");
            scrape_queries(queries, file)
                .await
                .expect("Failed to scrape queries");
        }
    }
}

async fn analyze(site: Site) {
    init();
    let mongodb_connection_url =
        std::env::var("MONGODB_CONNECTION_URL").expect("MONGODB_CONNECTION_URL not set");
    let database_name = std::env::var("DATABASE").expect("DATABASE not set");
    let db = connect(&mongodb_connection_url, &database_name).await;
    log::info!("Connected to database");

    let filename = site.filename();
    log::info!("Analyzing {}", filename);
    let file = File::open(filename)
        .await
        .expect("Failed to open json data file");
    let bytes = file.metadata().await.expect("Failed to get metadata").len();
    log::info!("File size: {} bytes", bytes);
    let data = tokio::fs::read_to_string(filename)
        .await
        .expect("Failed to read file");
    let data: Vec<JobPost> = serde_json::from_str(&data).expect("Failed to parse json");
    stream::iter(data)
        .map(create_job)
        .buffer_unordered(100)
        .for_each(|job| {
            let db = db.clone();
            async move {
                let job_json = to_string(&job).expect("Failed to serialize job");
                log::info!("Saving job: {}", job_json);
                let db_result = save_job(db, &job).await;
                match db_result {
                    Ok(rs) => log::info!("Saved job, id: {:?}", rs),
                    Err(e) => log::error!("Failed to save job: {}", e),
                }
            }
        })
        .await;
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let args = Cli::parse();
    let sites = args.site.iter().map(|s| Site::from(s.as_str()));
    match args.command {
        Commands::Scrape {} => stream::iter(sites).for_each(scrape).await,
        Commands::Analyze {} => stream::iter(sites).for_each(analyze).await,
    };
}
