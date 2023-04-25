use crate::cookies::encode_cookies;
use async_stream::stream;
use futures::{Future, Stream};
use reqwest::{
    header::{HeaderMap, HeaderValue, COOKIE},
    Client,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};
use thiserror::Error;

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct Filter {
    job_status: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobEntry {
    favorite: bool,
    seen: bool,
    hidden: bool,
    pub job: Job,
}

#[derive(Debug, Deserialize, Serialize)]
struct Language {
    title: String,
    rating: String,
    must_have: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Location {
    uuid: String,
    country_code: String,
    country: String,
    full_name: String,
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Skill {
    uuid: String,
    name: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub(crate) uuid: String,
    name: String,
    languages: Vec<Language>,
    seniorities: Vec<String>,
    management: bool,
    degree: Option<String>,
    freelancer: bool,
    willingness_to_travel: bool,
    contract_type: String,
    remote: bool,
    remote_type: Option<String>,
    salary_min: Option<u32>,
    salary_max: Option<u32>,
    currency: Option<String>,
    company: Company,
    locations: Vec<Location>,
    top_skills: Vec<Skill>,
    responsibles: Vec<Responsible>,
    primary_responsible: Responsible,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Company {
    name: String,
    company_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Responsible {
    name: String,
    gender: Option<String>,
    job_title: String,
    department: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Meta {
    page: Option<String>,
    per_page: u32,
    total_pages: u32,
    total_results: u32,
    pit_id: String,
    search_after: (String, String),
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ResponseBody {
    job_suggestions: Vec<JobEntry>,
    meta: Meta,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestBody {
    filters: Filter,
    #[serde(skip_serializing_if = "Option::is_none")]
    search_after: Option<(String, String)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pit_id: Option<String>,
}

pub async fn scrape(session_cookie_value: String) -> impl Stream<Item = crate::Job> {
    let mut cookies = HashMap::new();
    let session_cookie = String::from(urlencoding::encode(&session_cookie_value));
    cookies.insert("_instaffo_session".to_owned(), session_cookie);
    let mut headers = HeaderMap::new();
    headers.insert(
        COOKIE,
        HeaderValue::from_str(&encode_cookies(cookies.into_iter())).unwrap(),
    );
    let client = Client::builder().default_headers(headers).build().unwrap();
    let mut pit_id: Option<String> = None;
    let mut search_after: Option<(String, String)> = None;
    stream! {
        loop {
            let body = RequestBody {
                filters: Filter {
                    job_status: "open".to_owned(),
                },
                pit_id,
                search_after,
            };
            let resp = client
                .post("https://app.instaffo.com/candidate/api/v1/job_suggestions")
                .json(&body)
                .send()
                .await;
            let resp = match resp {
                    Ok(resp) => resp,
                    Err(e) => {
                        log::error!("Request failed: {}, stopping the scrape", e);
                        break;
                    },

                };
            if resp.status() != 200 {
                log::error!("Request not successful, status code: {}", resp.status());
                log::error!("Request not successful, body: {}", resp.text().await.unwrap_or("empty".to_owned()));
                break;
            }
            let resp_body: ResponseBody = match resp.json().await {
                    Ok(json_body) => json_body,
                    Err(e) => {
                        log::error!("Failed reading body from request: {}", e);
                        break;
                    },
                };
            pit_id = Some(resp_body.meta.pit_id);
            search_after = Some(resp_body.meta.search_after);
            for job_entry in resp_body.job_suggestions {
                yield crate::Job::Instaffo {
                    job_entry: Box::new(job_entry),
                };
            }
            log::info!("Successfully yielded all jobs from request, starting next loop iteration with pit_id: {:?}, search_after: {:?}", pit_id, search_after);
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use reqwest::{
        header::{HeaderMap, HeaderValue, COOKIE},
        Client,
    };
    use serde_json::{json, Value};

    use super::*;

    #[tokio::test]
    async fn test_request() {
        env_logger::init();
        let results = scrape("hC+7cYfHKxknVSUBImU7Wd+Jm3Ge480NPN2uh+FtpBX85AWGObfClwRrPPfL9ABr8yBfB2wsOVfok8uN9v7LKN0UIgDiBNabiBIKplQ0BhJZS+qqbprJwF7DPHiSmjGGSiu1A5Snj+6AT90REK2bhHGER2kDq73rngVeoC3bzc8Gxh2tqC044YCtooLLYtGCTvV083KKXbd68zaiofKO8Db8Sk1x+Mj+OsWqA5zT+0j7uqUR8e8esxE4BlLaNPytygWsYOrvOCeYc2SaDJ+Ozw3YMMtPqnY0xR7Q1KeZJ7ibajRpQvO0+cbfUjXPFwRrYKAEBA07E0ovItJ8t74Dfm5kmSI7W9MrC6y/gr8xteyyBmsy/h617URoT46fVfZqUNRbRBs6SepRrVci1jmdnV/dtvVm7Drg6310KtTwyX33loh76/QU31al5y0qoHA+fAQF4EKu8cgX--3EtkrU/+FvhAPBLp--X7z4dAW34dBAmAiKka9KCg==".to_owned()).await;
    }
}
