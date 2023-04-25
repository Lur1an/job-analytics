use std::{collections::HashSet, iter::Flatten};

use futures::{stream, Future, Stream, StreamExt};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

fn extract_job_id(job_url: &str) -> Option<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r".*-(\d+)\?.*").unwrap();
    }
    RE.captures(job_url)?.get(1).map(|m| m.as_str().to_owned())
}

fn job_search_url(query: &str, location: &str, offset: u32) -> String {
    return format!(
            "https://www.linkedin.com/jobs-guest/jobs/api/seeMoreJobPostings/search?location={}&keywords={}&start={}",
            location, urlencoding::encode(query), offset
        );
}

#[derive(Serialize, Deserialize, Debug)]
struct Job {}

async fn scrape_job_ids(
    client: Client,
    query: String,
    location: String,
) -> impl Stream<Item = Vec<String>> {
    let mut offset = 0;
    async_stream::stream! {
        loop {
            let url = job_search_url(&query, &location, offset);
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

pub async fn scrape_job(client: Client, id: String) -> Option<crate::Job> {
    todo!()
}

pub async fn scrape(
    queries: Vec<String>,
    locations: Vec<String>,
) -> impl Stream<Item = impl Future<Output = Option<crate::Job>>> {
    let client = Client::new();
    let product = queries.iter().flat_map(|query| {
        locations
            .iter()
            .map(move |location| (query.to_owned(), location.to_owned()))
    });
    let id_stream = stream::iter(product)
        .map(|(query, location)| scrape_job_ids(client.clone(), query, location))
        .buffer_unordered(20)
        .flatten()
        .collect::<Vec<_>>()
        .await;
    log::info!(
        "Found {} unique job ids, creating output stream of scraped jobs",
        id_stream.len()
    );
    let ids = id_stream.into_iter().flatten().collect::<HashSet<String>>();
    stream::iter(ids).map(move |job_id| scrape_job(client.clone(), job_id))
}

#[cfg(test)]
mod test {
    use std::{collections::HashSet, time::Duration};

    use lazy_static::lazy_static;
    use regex::Regex;
    use scraper::{Html, Selector};
    use thirtyfour::{
        prelude::{ElementQueryable, ElementWaitable},
        By, DesiredCapabilities, WebDriver,
    };
    use tokio::process::Command;

    fn extract_job_id(job_url: &str) -> Option<&str> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r".*-(\d+)\?.*").unwrap();
        }
        RE.captures(job_url)?.get(1).map(|m| m.as_str())
    }

    fn job_search_url(keywords: &str, location: &str, offset: u32) -> String {
        return format!(
            "https://www.linkedin.com/jobs-guest/jobs/api/seeMoreJobPostings/search?location={}&keywords={}&start={}",
            location, urlencoding::encode(keywords), offset
        );
    }

    #[tokio::test]
    async fn dev() {
        env_logger::init();
        let client = reqwest::Client::new();
        let job_search = job_search_url("Software Engineer Python", "Germany", 500);
        let response = client.get(job_search).send().await.unwrap();
        log::info!("Status: {}", response.status());
        let response = response.text().await.unwrap();
        let doc = Html::parse_document(&response);
        let selector = Selector::parse(".base-card__full-link").unwrap();
        let mut ids = Vec::new();
        for e in doc.select(&selector) {
            let url = e.value().attr("href").unwrap();
            log::info!("Found link: {:?}", url);
            let job_id = extract_job_id(url).unwrap();
            log::info!("Found job id: {}", job_id);
            ids.push(job_id);
            break;
        }
        for id in ids {
            let job_link = format!(
                "https://www.linkedin.com/jobs-guest/jobs/api/jobPosting/{}",
                id
            );
            log::info!("Sending GET to: {}", job_link);
            let response = client
                .get(job_link)
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap();
            let doc = Html::parse_document(&response);
            log::info!("Parsed document for job: {}", id);
            let org_selector = Selector::parse(".topcard__org-name-link").unwrap();
            let org_elem = doc.select(&org_selector).next().unwrap();
            let org_url = org_elem.value().attr("href").unwrap();
            let org_name: String = org_elem.text().filter(|c| !c.is_empty()).collect();
            let location_selector =
                Selector::parse("span.topcard__flavor.topcard__flavor--bullet").unwrap();
            let loc_elem = doc.select(&location_selector).next().unwrap();
            let loc_name: String = loc_elem.text().collect();
            let age_selector = Selector::parse("span.posted-time-ago__text").unwrap();
            let age: String = doc.select(&age_selector).next().unwrap().text().collect();
            let age = age.trim();
            let raw_data_selector = Selector::parse("div.description__text").unwrap();
            let raw_data: String = doc
                .select(&raw_data_selector)
                .next()
                .unwrap()
                .text()
                .collect();

            log::info!("Org url: {}", org_url);
            log::info!("Org name: {}", org_name);
            log::info!("Loc name: {}", loc_name.trim());
            log::info!("Post age: {}", age.trim());
            log::info!("Raw job description: {}", raw_data.trim());
            break;
        }
    }
    // #[tokio::test]
    // async fn dev() {
    //     env_logger::init();
    //     let mut chromedriver = Command::new("chromedriver").spawn().unwrap();
    //     let mut caps = DesiredCapabilities::chrome();
    //     // let _ = caps.add_chrome_arg("--headless");
    //     let driver = WebDriver::new("http://localhost:9515", caps).await.unwrap();
    //     driver
    //         .goto("https://www.linkedin.com/jobs/search?location=Germany")
    //         .await
    //         .unwrap();
    //     let cookie_button = driver
    //         .query(By::Css("[action-type='ACCEPT']"))
    //         .wait(Duration::from_secs(2), Duration::from_millis(100))
    //         .first()
    //         .await
    //         .unwrap();
    //     cookie_button.wait_until().clickable().await.unwrap();
    //     cookie_button.click().await.unwrap();
    //     log::info!("Clicked button");
    //     let selector = ".base-card__full-link";
    //     let mut links: HashSet<String> = HashSet::new();
    //     loop {
    //         let search_results = driver.find_all(By::Css(selector)).await.unwrap();
    //         log::info!("Found {} job cards", search_results.len());
    //         for result in search_results {
    //             let url = result.attr("href").await.unwrap().unwrap();
    //             links.insert(url);
    //         }
    //         driver
    //             .action_chain()
    //             .send_keys("PageDown")
    //             .perform()
    //             .await
    //             .unwrap();
    //         let mut keep = String::new();
    //         let _ = std::io::stdin()
    //             .read_line(&mut keep)
    //             .expect("Failed to read line");
    //         if keep.contains("no") {
    //             break;
    //         }
    //     }
    //     log::info!("Found {} links", links.len());
    //
    //     let mut _trash = String::new();
    //     std::io::stdin()
    //         .read_line(&mut _trash)
    //         .expect("Failed to read line");
    //     let _ = driver.quit().await;
    //     chromedriver.kill().await.unwrap();
    // }
}
