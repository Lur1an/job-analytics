use std::cmp::min;

use log;
use reqwest::{header::HeaderName, Client};
use serde::{Deserialize, Serialize};
use serde_json::json;

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
struct Job {
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
pub struct JobSearch {
    items: Vec<Job>,
    meta: MetaData,
}

fn job_search_url(offset: u32, results: u32, search: &str) -> String {
    format!(
        "https://www.xing.com/jobs/api/search?offset={}&limit={}&keywords={}",
        offset, results, search
    )
}

async fn scrape_job_search_page(
    client: &Client,
    offset: u32,
    results: u32,
    search: &str,
) -> Result<JobSearch> {
    let url = job_search_url(offset, results, search);
    log::info!(
        "requesting jobs from xing, offset: {}, search: {}",
        offset,
        search
    );
    let resp = client
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await?;
    log::info!("response status to job search: {}", resp.status());
    let job_search: JobSearch = resp.json().await?;
    Ok(job_search)
}

/// Scrapes all job entries for a given search term
///
/// # Errors
///
/// This function will return an error if:
/// - the request fails (status code != 200)
/// - the parsing fails
pub async fn scrape(keyword: &str) -> Result<Vec<Result<JobSearch>>> {
    let client = reqwest::Client::new();
    let results_per_page = 100;
    let first_page = scrape_job_search_page(&client, 0, results_per_page, keyword).await?;
    let last_page_index = first_page.meta.max_page;
    let mut results = Vec::with_capacity(first_page.meta.count as usize);
    results.push(Ok(first_page));
    for i in 1..min(last_page_index, 39) {
        let offset = i * results_per_page;
        let page = scrape_job_search_page(&client, offset, results_per_page, keyword).await;
        results.push(page);
    }
    Ok(results)
}

// test module
#[cfg(test)]
mod test {
    use super::*;
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }
    #[test]
    fn test_deserialize_job() {
        let job_json: &str = r#"{
                "id": 99865855,
                "scrambledId": "99865855.f81afe",
                "company": {
                    "name": "Dierks Business Consulting",
                    "link": "https://www.xing.com/pages/dierksbusinessconsulting",
                    "kununuData": {
                        "companyProfileUrl": "https://www.kununu.com/de/dierksbusinessconsulting",
                        "ratingAverage": 4.8,
                        "ratingCount": 64
                    }
                },
                "favoritePosting": null,
                "highlight": null,
                "isBookmarked": false,
                "isProjob": false,
                "link": "https://www.xing.com/jobs/nuernberg-senior-fullstack-software-entwickler-java-99865855?paging_context=search&search_query%5Blimit%5D=20&search_query%5Blocation%5D=Nuremberg%2C+Bavaria%2C+Germany&search_query%5Boffset%5D=0&search_query%5Bradius%5D=20",
                "location": "Nürnberg",
                "position": 0,
                "thumbnail": "https://www.xing.com/imagecache/public/scaled_original_image/eyJ1dWlkIjoiZWY5YjU1YmQtNzAyYy00MzdjLTlkZGEtNWNhYjA5NjQwMWJhIiwiYXBwX2NvbnRleHQiOiJlbnRpdHktcGFnZXMiLCJtYXhfd2lkdGgiOjE5MiwibWF4X2hlaWdodCI6MTkyfQ?signature=029eb95147e13ce6bb72c9da83d1cb0d2dc1b335b084dd7713c59a8e4419db11",
                "activatedAt": "2023-04-14T04:04:56Z",
                "path": "/jobs/nuernberg-senior-fullstack-software-entwickler-java-99865855?paging_context=search&search_query%5Blimit%5D=20&search_query%5Blocation%5D=Nuremberg%2C+Bavaria%2C+Germany&search_query%5Boffset%5D=0&search_query%5Bradius%5D=20",
                "slug": "nuernberg-senior-fullstack-software-entwickler-java-99865855",
                "title": "Senior Fullstack Software-Entwickler Java (m/w/d)",
                "trackingToken": "2d6f0967da9411edb24a2e26756c7123.0.99865855"
            }"#;
        let job: Job = serde_json::from_str(job_json).unwrap();
        assert_eq!(job.id, 99865855);
    }

    #[tokio::test]
    async fn test_get_and_deserialize_job_search() {
        init();
        let _ = scrape_job_search_page(&reqwest::Client::new(), 0, 100, None)
            .await
            .expect("Request failed");
    }

    #[tokio::test]
    async fn test_scrape_all() {
        init();
        let results = scrape().await;
    }
}
