use tokio::fs::File;
use crate::Target;

pub async fn scrape(site: Target) {
    match site {
        Target::Xing => {
            todo!()
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
