use serde::{Deserialize, Serialize};

pub mod db;

#[derive(Deserialize, Serialize)]
pub enum Site {
    Xing,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SalaryRange {
    min: u32,
    max: u32,
}

impl SalaryRange {
    pub fn new(min: u32, max: u32) -> Self {
        Self { min, max }
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

#[derive(Deserialize, Serialize)]
pub enum ExperienceLevel {
    Internship,
    Junior,
    Mid,
    Senior,
    Lead,
}
/// The details of a job post
/// This data is extracted through ChatGPT api, by using raw job data provided
#[derive(Deserialize, Serialize)]
pub struct JobDetails {
    requirements: Vec<String>,
    technologies: Vec<String>,
    programming_languages: Vec<String>,
    salary_forecast: Option<SalaryRange>,
    experience_level: ExperienceLevel,
}

impl JobDetails {
    pub fn new(
        requirements: Vec<String>,
        technologies: Vec<String>,
        programming_languages: Vec<String>,
        salary_forecast: Option<SalaryRange>,
        experience_level: ExperienceLevel,
    ) -> Self {
        Self {
            requirements,
            technologies,
            programming_languages,
            salary_forecast,
            experience_level,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct JobPost {
    title: String,
    link: String,
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
        company: Company,
        location: String,
        raw_data: Option<String>,
        posting_date: Option<String>,
    ) -> Self {
        Self {
            title,
            link,
            company,
            location,
            raw_data,
            posting_date,
        }
    }
}
