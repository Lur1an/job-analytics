use futures::{stream, StreamExt};
use job_analyzer::JobPost;
use std::collections::HashSet;
use std::{cmp::min, hash::Hash};

use log;
use reqwest::Client;

use scraper::Html;
use scraper::Selector;
use serde::{Deserialize, Serialize};

use crate::api::Error;

type Result<T> = std::result::Result<T, crate::api::Error>;

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
pub struct XingJob {
    id: u32,
    scrambled_id: String,
    company: Company,
    favorite_posting: Option<String>,
    highlight: Option<String>,
    is_bookmarked: bool,
    is_projob: bool,
    link: String,
    location: String,
    position: u32,
    thumbnail: Option<String>,
    activated_at: Option<String>,
    path: Option<String>,
    slug: Option<String>,
    title: String,
    tracking_token: Option<String>,
}

impl PartialEq for XingJob {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for XingJob {}
impl Hash for XingJob {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.scrambled_id.hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct MetaData {
    count: u32,
    #[serde(alias = "currentPage")]
    current_page: u32,
    #[serde(alias = "maxPage")]
    max_page: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse {
    pub items: Vec<XingJob>,
    meta: MetaData,
}

fn job_search_url(offset: u32, results: u32, search: &str) -> String {
    format!(
        "https://www.xing.com/jobs/api/search?employmentType=FULL_TIME.ef2fe9&offset={}&limit={}&keywords={}",
        offset, results, search
    )
}

async fn scrape_job_search_page(
    client: &Client,
    offset: u32,
    results: u32,
    search: &str,
) -> Result<ApiResponse> {
    let url = job_search_url(offset, results, search);
    log::info!(
        "requesting jobs from xing, offset: {}, search: {}",
        offset,
        search
    );
    let resp = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await?;
    if !resp.status().is_success() {
        log::error!(
            "failed to retrieve results for offset{}, search: {}",
            offset,
            search
        );
        return Err(crate::api::Error::RequestNotOk(url));
    }

    log::info!(
        "successfully retrieved results for offset{}, search: {}",
        offset,
        search
    );
    let job_search: ApiResponse = resp.json().await?;
    Ok(job_search)
}

pub async fn scrape_job_search_batch(
    start: u32,
    end: u32,
    search: String,
    results_per_page: u32,
    client: Client,
) -> Vec<Result<ApiResponse>> {
    let mut results = Vec::with_capacity((end - start) as usize);
    for page in start..end {
        let offset = page * results_per_page;
        let page = scrape_job_search_page(&client, offset, results_per_page, search.as_str()).await;
        results.push(page);
    }
    results
}

pub async fn scrape(
    client: Client,
    keyword: String,
    workers: u32,
) -> Result<Vec<Result<ApiResponse>>> {
    let results_per_page = 100;
    let first_page = scrape_job_search_page(&client, 0, results_per_page, &keyword).await?;
    let results_count = min(first_page.meta.count, 1000);
    let mut results = Vec::with_capacity(results_count as usize);
    let last_page_index = min(first_page.meta.max_page, results_count / results_per_page);
    results.push(Ok(first_page));

    let mut handles = Vec::with_capacity(workers as usize);
    let pages_per_worker = last_page_index / workers;
    let remainder = last_page_index % workers;
    // Start threads
    for w in 0..workers {
        let client = client.clone();
        let start = pages_per_worker * w + 1;
        let mut end = start + pages_per_worker;
        if w == workers - 1 && remainder > 0 {
            end += remainder - 1;
        }
        let handle = tokio::spawn(scrape_job_search_batch(
            start,
            end,
            String::from(&keyword),
            results_per_page,
            client,
        ));
        handles.push(handle);
    }
    futures::future::join_all(handles)
        .await
        .into_iter()
        .flatten()
        .into_iter()
        .flatten()
        .for_each(|page| results.push(page));
    Ok(results)
}

pub async fn scrape_queries(queries: Vec<String>) -> Vec<JobPost> {
    let mut handles = Vec::with_capacity(queries.len());
    let client = Client::new();
    queries.into_iter().for_each(|query| {
        let join_handle = tokio::spawn(scrape(client.clone(), query, 2));
        handles.push(join_handle);
    });
    let results = futures::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(|x| x.ok())
        .filter_map(|x| x.ok())
        .flatten()
        .filter_map(Result::ok)
        .map(|job_search| job_search.items)
        .flatten()
        .collect::<HashSet<_>>();

    stream::iter(results)
        .map(|xing_job| convert_xing_job(client.clone(), xing_job))
        .buffer_unordered(100)
        .collect::<Vec<JobPost>>()
        .await
}

async fn convert_xing_job(client: Client, job: XingJob) -> JobPost {
    let job_content = scrape_raw_job_content(client, &job.link).await;
    JobPost::new(
        job.title,
        job.link,
        job_analyzer::Company::new(
            job.company.name,
            job.company.link,
            job.company.kununu_data.map(|kd| {
                job_analyzer::KununuData::new(
                    kd.company_profile_url,
                    kd.rating_average,
                    kd.rating_count,
                )
            }),
        ),
        job.location,
        job_content.ok(),
        job.activated_at,
    )
}
pub async fn scrape_raw_job_content(client: Client, job_url: &str) -> Result<String> {
    let job_url = job_url;
    let html = client.get(job_url).send().await?.text().await?;
    let doc = Html::parse_document(&html);
    let job_data_selector = Selector::parse(
        ".styles-grid-gridContainer-cec162b7.styles-grid-standardGridContainer-cfa898d5",
    )
    .unwrap();
    let job_data = doc
        .select(&job_data_selector)
        .next()
        .ok_or(Error::ContentNotFound("Job Posting data"))?
        .text();

    let salary_selector = Selector::parse(r#"[data-cy="posting-salary"]"#).unwrap();
    let salary_data = doc
        .select(&salary_selector)
        .next()
        .ok_or(Error::ContentNotFound("Salary"))?
        .text();
    let raw_data = salary_data.chain(job_data).collect::<String>();
    return Ok(raw_data);
}

// test module
#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_get_and_deserialize_job_search() {
        let _ = scrape_job_search_page(&reqwest::Client::new(), 0, 100, "Software")
            .await
            .expect("Request failed");
    }

    #[tokio::test]
    async fn test_scrape_with_query() {
        let query = "React Frontend Engineer";
        let _results = scrape(Client::new(), query.to_owned(), 2)
            .await
            .expect("Scraping outer function should not fail");
    }

    #[tokio::test]
    async fn test_scrape_queries_and_convert_to_job_posting() {
        let queries = vec!["Svelte Engineer", "React Engineer"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>();
        let _results = scrape_queries(queries).await;
    }
    #[tokio::test]
    async fn test_parse_html_for_job_posting() {
        let job_url = "https://www.xing.com/jobs/nuernberg-anwendungsentwickler-java-98960724";
        let data = scrape_raw_job_content(Client::new(), job_url).await;
        assert!(data.is_ok());
    }
}
