pub(crate) mod cookies;
pub mod instaffo;
pub mod linkedin;
pub mod xing;

use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Job {
    Xing {
        job: Box<xing::types::Job>,
        raw_data: Option<String>,
    },
    Instaffo {
        job: Box<instaffo::Job>,
    },
    Linkedin {
        job: Box<linkedin::Job>,
    },
    Stepstone {},
    Glassdoor {},
    Indeed {},
}

impl Hash for Job {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Job::Xing { job, .. } => {
                job.hash(state);
            }
            Job::Linkedin { job } => todo!(),
            Job::Stepstone {} => todo!(),
            Job::Glassdoor {} => todo!(),
            Job::Indeed {} => todo!(),
            Job::Instaffo { job: job_entry } => {
                job_entry.job.uuid.hash(state);
                "instaffo".hash(state);
            }
        }
    }
}
