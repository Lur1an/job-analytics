pub mod xing;
use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Job {
    Xing { job: Box<xing::types::Job>, raw_data: Option<String> },
    Linkedin {},
    Stepstone {},
    Glassdoor {},
}
