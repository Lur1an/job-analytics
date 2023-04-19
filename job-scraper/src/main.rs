use lazy_static::lazy_static;
use job_scraper::xing;
use std::time::Duration;
use thirtyfour::prelude::*;
use tokio::time::sleep;

const SEARCH_QUERIES: [&str; 8] = [
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
    env_logger::init();
    println!("Hello, world!");
}
