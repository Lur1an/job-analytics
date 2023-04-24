mod analyze;
mod scrape;

use clap::{Parser, Subcommand};
use dotenv::dotenv;
use futures::{stream, StreamExt};
use ai_analyzer::{
    db::{connect, save_job},
    openai_analyzer::{create_job, init},
};
use job_scraper::xing::scrape_queries;
use serde_json::to_string;
use tokio::fs::File;
use ai_analyzer::types::{JobPost, Site};

enum Target {
    Xing,
    Linkedin,
    Stepstone,
    Glassdoor,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// List of job-providers to execute the command against
    #[clap(long)]
    site: Vec<Target>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Scrape {},
    Analyze {},
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let args = Cli::parse();
    match args.command {
        Commands::Scrape {} => stream::iter(args.site).for_each(scrape::scrape).await,
        Commands::Analyze {} => stream::iter(args.site).for_each(analyze::analyze).await,
    };
}
