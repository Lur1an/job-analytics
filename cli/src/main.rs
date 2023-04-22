use clap::{Parser, Subcommand};
use dotenv::dotenv;
use job_scraper::xing::scrape_queries;
use serde_json::to_string;
use tokio::fs::File;

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
const DEFAULT_SEARCH_QUERIES: [&str; 8] = [
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
            let results = scrape_queries(queries).await;
            let json = to_string(&results).expect("Failed to serialize results");
        }
        Commands::Analyze {} => todo!(),
    };
}
