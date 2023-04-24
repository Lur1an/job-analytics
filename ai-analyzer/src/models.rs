use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub enum Site {
    Xing,
}

impl Site {
    pub fn filename(&self) -> &'static str {
        match self {
            Site::Xing => "xing.json",
        }
    }
}

impl From<&str> for Site {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "xing" => Site::Xing,
            _ => panic!("Unknown site"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KununuData {
    company_profile_url: String,
    rating_average: f32,
    rating_count: u32,
}

impl KununuData {
    pub fn new(company_profile_url: String, rating_average: f32, rating_count: u32) -> Self {
        Self {
            company_profile_url,
            rating_average,
            rating_count,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Company {
    name: String,
    link: Option<String>,
    kununu_data: Option<KununuData>,
}

impl Company {
    pub fn new(name: String, link: Option<String>, kununu_data: Option<KununuData>) -> Self {
        Self {
            name,
            link,
            kununu_data,
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub enum ExperienceLevel {
    Junior,
    Mid,
    Senior,
    Lead,
}

/// The details of a job post
/// This data is extracted through ChatGPT api, by using raw job data provided
#[derive(Deserialize, Serialize, Debug)]
pub struct JobDetails {
    requirements: Vec<String>,
    technologies: Vec<String>,
    benefits: Vec<String>,
    programming_languages: Vec<String>,
    salary_forecast: Option<(u32, u32)>,
    experience_level: ExperienceLevel,
    application_url: Option<String>,
}

impl JobDetails {
    pub fn new(
        requirements: Vec<String>,
        technologies: Vec<String>,
        benefits: Vec<String>,
        programming_languages: Vec<String>,
        salary_forecast: Option<(u32, u32)>,
        experience_level: ExperienceLevel,
        application_url: Option<String>,
    ) -> Self {
        Self {
            requirements,
            technologies,
            programming_languages,
            salary_forecast,
            experience_level,
            application_url,
            benefits,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct JobPost {
    title: String,
    link: String,
    site: Site,
    company: Company,
    location: String,
    /// This data contains the raw-form of the content of the job post
    /// used by GPT endpoints to compute JobDetails. Might not be avaiable or failed to be extracted
    pub raw_data: Option<String>,
    /// formatted as yyyy-mm-dd
    posting_date: Option<String>,
}

impl JobPost {
    pub fn new(
        title: String,
        link: String,
        site: Site,
        company: Company,
        location: String,
        raw_data: Option<String>,
        posting_date: Option<String>,
    ) -> Self {
        Self {
            title,
            link,
            site,
            company,
            location,
            raw_data,
            posting_date,
        }
    }
}
