enum JobSite {
    Xing,
    Linkedin,
}

struct Range {
    min: u32,
    max: u32,
}

struct Company {
    name: String,
    link: Option<String>,
}

pub struct JobDetails {
    requirements: Vec<String>,
    technologies: Vec<String>,
    programming_languages: Vec<String>,
    salary_forecast: Option<Range>,
}

pub struct JobPost {
    title: String,
    site: JobSite,
    url: String,
    company: Company,
    location: String,
}
fn main() {
    println!("Hello, world!");
}
