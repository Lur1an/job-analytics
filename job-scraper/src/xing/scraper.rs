use async_stream::stream;
use futures::Stream;
use std::cmp::min;
use std::collections::HashSet;
use std::future::Future;

use log;
use reqwest::Client;

use scraper::Html;
use scraper::Selector;
use serde::{Deserialize, Serialize};

use crate::xing::types::Job;
use crate::xing::Error;
use crate::xing::Result;

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
struct ApiResponse {
    items: Vec<Job>,
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
    log::debug!(
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
        let error_body = resp.text().await;
        log::error!(
            "failed to retrieve results for offset: {}, search: {}, error resp body: {:?}",
            offset,
            search,
            error_body,
        );
        return Err(Error::RequestNotOk(url));
    }

    log::debug!(
        "successfully retrieved results for offset: {}, search: {}",
        offset,
        search
    );
    let job_search: ApiResponse = resp.json().await?;
    Ok(job_search)
}

async fn scrape_job_search_batch(
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

async fn scrape_api(
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
            end = end - 1 + remainder;
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

/// Scrape all jobs for given queries
/// Results are buffered into the tokin::fs::File provided
pub async fn scrape_queries(
    queries: Vec<String>,
) -> impl Stream<Item = impl Future<Output = crate::Job>> {
    let mut handles = Vec::with_capacity(queries.len());
    let client = Client::new();
    queries.into_iter().for_each(|query| {
        let join_handle = tokio::spawn(scrape_api(client.clone(), query, 2));
        handles.push(join_handle);
    });
    let results = futures::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter_map(Result::ok)
        .flatten()
        .filter_map(Result::ok)
        .flat_map(|job_search| job_search.items)
        .collect::<HashSet<_>>();

    log::info!(
        "found {} unique jobs, scraping page data for each",
        results.len()
    );

    let job_stream = stream! {
        for job in results {
            let job = convert(client.clone(), job);
            yield job;
        }
    };
    job_stream
}

/// scrape the raw data from the job posting page
/// then convert it to a JobPost
async fn convert(client: Client, job: Job) -> crate::Job {
    let job_content = scrape_job_content(client, &job.link).await;
    let job_content = match job_content {
        Ok(content) => {
            log::debug!("scraped job content for {}", job.link);
            Some(content)
        }
        Err(e) => {
            log::error!(
                "failed to scrape job content for url: {}, error: {}",
                job.link,
                e
            );
            None
        }
    };
    crate::Job::Xing {
        job: Box::new(job),
        raw_data: job_content,
    }
}

async fn scrape_job_content(client: Client, job_url: &str) -> Result<String> {
    let job_url = job_url;
    let resp = client.get(job_url).send().await?;
    let html = resp.text().await?;
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
    use futures::{stream, StreamExt};
    use tokio::pin;
    use tokio::time::Instant;

    #[tokio::test]
    async fn test_get_and_deserialize_job_search() {
        let _ = scrape_job_search_page(&Client::new(), 0, 100, "Software")
            .await
            .expect("Request failed");
    }

    #[tokio::test]
    async fn test_scrape_with_query() {
        let query = "React Frontend Engineer";
        let results = scrape_api(Client::new(), query.to_owned(), 2).await;
        assert!(results.is_ok(), "Failed to scrape with query: {}", query);
    }

    #[tokio::test]
    async fn test_scrape_stream_api() {
        env_logger::init();
        let queries = vec!["Svelte".to_owned(), "Rust".to_owned()];
        let stream = scrape_queries(queries).await.buffer_unordered(400);
        pin!(stream);
        let mut job_count = 0;
        while let Some(_) = stream.next().await {
            job_count += 1;
        }
        assert!(job_count > 0, "Failed to scrape any jobs");
    }

    #[tokio::test]
    async fn test_parse_html_for_job_posting() {
        let job_url = "https://www.xing.com/jobs/nuernberg-anwendungsentwickler-java-98960724";
        let data = scrape_job_content(Client::new(), job_url).await;
        assert!(data.is_ok(), "Failed to parse html");
    }
}
