use clap::{Parser, Subcommand};
use dotenv::dotenv;
use job_scraper::xing::scrape_queries;
use serde_json::to_string;
use tokio::{fs::File, io::AsyncWriteExt};

/// Simple program to greet a person
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
const DEFAULT_SEARCH_QUERIES: [&str; 13] = [
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

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let args = Cli::parse();
    match args.command {
        Commands::Scrape {} => {
            let queries = DEFAULT_SEARCH_QUERIES
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>();
            let file = File::create("xing.json")
                .await
                .expect("Failed to create file");
            scrape_queries(queries, file)
                .await
                .expect("Failed to scrape queries");
        }
        Commands::Analyze {} => todo!(),
    };
}
