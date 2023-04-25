pub(crate) mod cookies;
pub mod instaffo;
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
    Linkedin {},
    Stepstone {},
    Glassdoor {},
}

impl Hash for Job {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Job::Xing { job, .. } => {
                job.hash(state);
            }
            Job::Linkedin {} => todo!(),
            Job::Stepstone {} => todo!(),
            Job::Glassdoor {} => todo!(),
        }
    }
}
