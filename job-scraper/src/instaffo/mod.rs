use crate::cookies::combine_cookies;
use async_stream::stream;
use futures::{Future, Stream};
use reqwest::{
    header::{HeaderMap, HeaderValue, COOKIE},
    Client,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, time::Duration};
use thiserror::Error;
use tokio::{io::AsyncWriteExt, time::sleep};

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct Filter {
    job_status: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    favorite: bool,
    seen: bool,
    hidden: bool,
    pub job: JobData,
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
pub struct JobData {
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
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Company {
    name: String,
    company_type: String,
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
    job_suggestions: Vec<Job>,
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
        HeaderValue::from_str(&combine_cookies(cookies.into_iter())).unwrap(),
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
            let resp_body = match resp.text().await {
                    Ok(body) => body,
                    Err(e) => {
                        log::error!("Failed reading body from request: {}", e);
                        break;
                    },
            };
            let resp_body: ResponseBody = match serde_json::from_str(&resp_body) {
                    Ok(json_body) => json_body,
                    Err(e) => {
                        log::error!("Failed serializing response body: {}", e);
                        log::info!("Writing response body to instaffo.json");
                        let mut file = tokio::fs::File::create("instaffo.json").await.expect("Couldn't create instaffo.json");
                        file.write_all(resp_body.as_bytes()).await.expect("Couldn't write to instaffo.json");
                        break;
                    },
                };
            pit_id = Some(resp_body.meta.pit_id);
            search_after = Some(resp_body.meta.search_after);
            for job_entry in resp_body.job_suggestions {
                yield crate::Job::Instaffo {
                    job: Box::new(job_entry),
                };
            }
            log::info!("Successfully yielded all jobs from request, starting next loop iteration with pit_id: {:?}, search_after: {:?}", pit_id, search_after);
            sleep(Duration::from_secs(5)).await;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use reqwest::{
        header::{HeaderMap, HeaderValue, COOKIE},
        Client,
    };
    use serde_json::{json, Value};
    use std::collections::HashMap;
}
