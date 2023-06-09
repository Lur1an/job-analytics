use chrono::{Duration, Utc};
use futures::{stream, Future, Stream, StreamExt};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::header::{
    HeaderValue, ACCEPT, ACCEPT_LANGUAGE, CACHE_CONTROL, CONNECTION, UPGRADE_INSECURE_REQUESTS,
    USER_AGENT,
};
use reqwest::{header::HeaderMap, Client};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, iter::Flatten};
use tokio::sync::Mutex;
use tokio::time::sleep;

fn extract_job_id(job_url: &str) -> Option<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r".*-(\d+)\?.*").unwrap();
    }
    RE.captures(job_url)?.get(1).map(|m| m.as_str().to_owned())
}

fn job_search_url(query: &str, location: &str, offset: u32) -> String {
    return format!(
            "https://www.linkedin.com/jobs-guest/jobs/api/seeMoreJobPostings/search?location={}&keywords={}&start={}&f_TPR=r604800",
            location, urlencoding::encode(query), offset
        );
}

lazy_static! {
    static ref SCRAPED_IDS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

async fn scrape_job_ids(
    client: Client,
    query: String,
    location: String,
) -> impl Stream<Item = Vec<String>> {
    let mut offset = 0;
    async_stream::stream! {
        loop {
            let url = job_search_url(&query, &location, offset);
            log::info!("GET {}", url);
            let resp = match client.get(&url).send().await {
                Ok(resp) => resp,
                Err(err) => {
                    log::error!("Error while sending request: {}, url: {}", err, url);
                    break;
                }
            };
            let status = resp.status();
            let body = match resp.text().await {
                Ok(resp) => resp,
                Err(err) => {
                    log::error!("Error while streaming response body: {}", err);
                    break;
                }
            };

            if status == 429 {
                log::info!("Rate limit exceeded (Status 429), sleeping...");
                sleep(Duration::seconds(5).to_std().unwrap()).await;
                continue;
            }

            if status != 200 {
                log::error!("Request not successful, status code: {}, body: {}", status, body);
                break;
            }

            let selector = Selector::parse(".base-card__full-link").unwrap();
            let doc = Html::parse_document(&body);
            let ids = doc.select(&selector).map(|el| {
                el.value().attr("href").and_then(extract_job_id)
            })
                .filter(Option::is_some)
                .map(Option::unwrap)
                .collect::<Vec<String>>();
            yield ids;
            offset += 25;
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Company {
    pub name: Option<String>,
    link: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Job {
    pub linkedin_id: String,
    pub title: Option<String>,
    location: Option<String>,
    pub company: Company,
    posting_date: Option<chrono::DateTime<Utc>>,
    raw_data: Option<String>,
    criteria: JobCriteria,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JobCriteria {
    seniority: Option<String>,
    employment_type: Option<String>,
    job_function: Option<String>,
    industries: Option<String>,
}

fn try_convert_age(age: &str) -> Option<chrono::DateTime<Utc>> {
    let elems = age.split(" ").collect::<Vec<_>>();
    let unit = *elems.get(1)?;
    let parse_amount: i64 = (*elems.get(0)?).parse().ok()?;
    let mut today = Utc::now();
    match unit.to_lowercase().as_ref() {
        "days" => today -= Duration::days(parse_amount),
        "weeks" => today -= Duration::weeks(parse_amount),
        "years" => today -= Duration::days(parse_amount * 365),
        _ => {}
    };
    Some(today)
}

pub async fn scrape_job(client: Client, id: String) -> Option<crate::Job> {
    let job_link = format!(
        "https://www.linkedin.com/jobs-guest/jobs/api/jobPosting/{}",
        id
    );
    log::info!("Sending GET to: {}", job_link);
    let response = match client.get(job_link).send().await {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Failed to send request: {}", e);
            return None;
        }
    };
    let status = response.status();
    let body = match response.text().await {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to read response body: {}", e);
            return None;
        }
    };

    if status == 429 {
        log::info!("Rate limit exceeded (Status 429), sleeping...");
        sleep(Duration::seconds(5).to_std().unwrap()).await;
        return None;
    }

    if status != 200 {
        log::error!(
            "Request not successful, status code: {}, body: {}",
            status,
            body
        );
        return None;
    }
    let doc = Html::parse_document(&body);
    log::info!("Parsed document for job: {}", id);

    let title_selector = Selector::parse(".top-card-layout__title").unwrap();
    let title = doc
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>());

    let org_selector = Selector::parse(".topcard__org-name-link").unwrap();
    let (org_url, org_name) = doc
        .select(&org_selector)
        .next()
        .map(|el| {
            let org_url = el.value().attr("href").map(String::from);
            let org_name: String = el
                .text()
                .filter(|c| !c.is_empty())
                .map(str::trim)
                .collect::<String>();
            (org_url, Some(org_name))
        })
        .unwrap_or((None, None));

    let location_selector =
        Selector::parse("span.topcard__flavor.topcard__flavor--bullet").unwrap();
    let loc_name: Option<String> = doc
        .select(&location_selector)
        .next()
        .map(|el| el.text().map(str::trim).collect());
    let age_selector = Selector::parse("span.posted-time-ago__text").unwrap();
    let age = doc
        .select(&age_selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_owned())
        .map(|s| try_convert_age(&s))
        .flatten();
    let raw_data_selector = Selector::parse("div.description__text").unwrap();
    let raw_data: Option<String> = doc
        .select(&raw_data_selector)
        .next()
        .map(|el| el.text().filter(|c| !c.is_empty()).map(str::trim).collect());
    let criteria_selector = Selector::parse(".description__job-criteria-text").unwrap();
    let criteria: Vec<String> = doc
        .select(&criteria_selector)
        .map(|el| el.text().map(str::trim).collect())
        .collect();
    let seniority = criteria.get(0).map(String::to_owned);
    let employment_type = criteria.get(1).map(String::to_owned);
    let job_function = criteria.get(2).map(String::to_owned);
    let industries = criteria.get(3).map(String::to_owned);
    let job = Box::new(Job {
        linkedin_id: id,
        title,
        location: loc_name,
        company: Company {
            name: org_name,
            link: org_url,
        },
        posting_date: age,
        raw_data,
        criteria: JobCriteria {
            seniority,
            employment_type,
            job_function,
            industries,
        },
    });
    Some(crate::Job::Linkedin { job })
}

pub async fn scrape(
    queries: Vec<String>,
    locations: Vec<String>,
) -> impl Stream<Item = Option<crate::Job>> {
    log::info!("creating client and producing query products");
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9"));
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("max-age=0"));
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
    headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
    headers.insert(UPGRADE_INSECURE_REQUESTS, HeaderValue::from_static("1"));
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36"));
    let client = Client::builder().default_headers(headers).build().unwrap();
    let product = queries
        .iter()
        .flat_map(|query| {
            locations
                .iter()
                .map(move |location| (query.to_owned(), location.to_owned()))
        })
        .collect::<Vec<(String, String)>>();
    let mut scraped_ids: HashSet<String> = HashSet::new();
    async_stream::stream! {
        for (query, location) in product {
            let id_chunks = scrape_job_ids(client.clone(), query, location).await;
            tokio::pin!(id_chunks);
            while let Some(ids) = id_chunks.next().await {
                for id in ids {
                    if scraped_ids.contains(&id) {
                        continue;
                    }
                    scraped_ids.insert(id.clone());
                    let job = scrape_job(client.clone(), id).await;
                    yield job;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_try_convert_age() {
        let age = "1 day ago";
        let res = super::try_convert_age(age);
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(res < chrono::Utc::now());
    }
}
