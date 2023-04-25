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
        log::info!("Start");
        let body = json!({
                    "filters": {
                        "jobStatus": "open",
                        "remote": true
                    }
        });
        let mut headers = HeaderMap::new();
        let mut cookies: HashMap<&str, &str> = HashMap::new();
        cookies.insert("_instaffo_session", "Session Cookie");
        let cookies = cookies
            .into_iter()
            .map(|(k, v)| (k, urlencoding::encode(v)))
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(";");
        headers.insert(COOKIE, HeaderValue::from_str(&cookies).unwrap());
        let client = Client::builder().default_headers(headers).build().unwrap();
        let resp = client
            .post("https://app.instaffo.com/candidate/api/v1/job_suggestions")
            .json::<Value>(&body)
            .send()
            .await;
        let resp_body = resp.unwrap().json::<Value>().await.unwrap();
        log::info!("Body: {}", resp_body);
    }
}
