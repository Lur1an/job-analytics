use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct KununuData {
    company_profile_url: String,
    rating_average: f32,
    rating_count: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Company {
    name: String,
    link: Option<String>,
    kununu_data: Option<KununuData>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    id: u32,
    scrambled_id: String,
    company: Company,
    favorite_posting: Option<String>,
    highlight: Option<String>,
    is_bookmarked: bool,
    is_projob: bool,
    pub(crate) link: String,
    location: String,
    position: u32,
    thumbnail: Option<String>,
    activated_at: Option<chrono::DateTime<Utc>>,
    path: Option<String>,
    slug: Option<String>,
    title: String,
    tracking_token: Option<String>,
}

impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Job {}

impl Hash for Job {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        "xing".hash(state);
    }
}
