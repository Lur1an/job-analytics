use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CommandArgs {
    /// The name of the person to greet
    #[clap(long)]
    site: Option<String>,
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
fn main() {
    let args = CommandArgs::parse();
    println!("Hello {:?}!", args.site)
}
