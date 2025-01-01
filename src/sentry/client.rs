use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;

use crate::handlers::status::Status;

pub struct SentryClient {
    http_client: Arc<reqwest::Client>,
    host: String,
}

impl SentryClient {
    pub fn new(host: String, http_client: Arc<reqwest::Client>) -> Self {
        Self {
            http_client,
            host,
        }
    }

    pub async fn get_status(&self) -> Status {
        let url = format!("{}/status", self.host);
        let res = self.http_client.get(&url).send().await;
        if res.is_err() {
            return Status::unreachable();
        }
        let res = res.unwrap();
        if !res.status().is_success() {
            Status::unreachable()
        } else {
            return Status::deserialize(res.json::<Value>().await.unwrap()["status"].clone()).unwrap_or(Status::unreachable());
        }
    }
}