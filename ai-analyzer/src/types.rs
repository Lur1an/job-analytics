use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub enum ExperienceLevel {
    Junior,
    Mid,
    Senior,
    Lead,
}

#[derive(Deserialize, Debug, Serialize)]
pub enum Workplace {
    Remote,
    Onsite,
    Hybrid,
}

/// The details of a job post, extracted through AI analysis
#[derive(Deserialize, Serialize, Debug)]
pub struct JobDetails {
    requirements: Vec<String>,
    technologies: Vec<String>,
    benefits: Vec<String>,
    programming_languages: Vec<String>,
    salary_forecast: Option<(u32, u32)>,
    requires_degree: Option<String>,
    experience_level: Option<ExperienceLevel>,
    application_url: Option<String>,
    workplace: Option<Workplace>,
}

impl JobDetails {
    pub fn new(
        requirements: Vec<String>,
        technologies: Vec<String>,
        benefits: Vec<String>,
        programming_languages: Vec<String>,
        salary_forecast: Option<(u32, u32)>,
        requires_degree: Option<String>,
        experience_level: Option<ExperienceLevel>,
        application_url: Option<String>,
        workplace: Option<Workplace>,
    ) -> Self {
        Self {
            requirements,
            technologies,
            programming_languages,
            salary_forecast,
            experience_level,
            application_url,
            benefits,
            workplace,
            requires_degree,
        }
    }
}
