mod analyze;
mod scrape;

use clap::{Parser, Subcommand};
use dotenv::dotenv;
use futures::{stream, StreamExt};

#[derive(Clone)]
pub enum Target {
    Xing,
    Linkedin,
    Stepstone,
    Glassdoor,
    Instaffo,
}

impl From<String> for Target {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "xing" => Target::Xing,
            "linkedin" => Target::Linkedin,
            "stepstone" => Target::Stepstone,
            "glassdoor" => Target::Glassdoor,
            "instaffo" => Target::Instaffo,
            _ => panic!("Unknown target: {}", s),
        }
    }
}

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

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let args = Cli::parse();
    let sites = args.site.into_iter().map(Target::from);
    match args.command {
        Commands::Scrape {} => stream::iter(sites).for_each(scrape::scrape).await,
        Commands::Analyze {} => stream::iter(sites).for_each(analyze::analyze).await,
    };
}
