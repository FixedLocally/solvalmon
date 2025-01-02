use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;

use crate::handlers::{set_identity::{self, IdentityVariant}, status::Status, tower::PostTower};

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
        Status {
            hostname: self.host.clone(),
            ..self._get_status().await
        }
    }

    async fn _get_status(&self) -> Status {
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

    pub async fn get_tower(&self) -> Option<PostTower> {
        let url = format!("{}/tower", self.host);
        let res = self.http_client.get(&url).send().await;
        if res.is_err() {
            return None;
        }
        let res = res.unwrap();
        if !res.status().is_success() {
            None
        } else {
            Some(PostTower::deserialize(&res.json::<Value>().await.unwrap()["tower"]).unwrap())
        }
    }

    pub async fn post_tower(&self, tower: String) -> bool {
        let url = format!("{}/tower", self.host);
        let res = self.http_client.post(&url).json(&PostTower { tower }).send().await;
        if res.is_err() {
            return false;
        }
        res.unwrap().status().is_success()
    }

    pub async fn set_identity(&self, identity: IdentityVariant) -> bool {
        let url = format!("{}/set_identity", self.host);
        let res = self.http_client.post(&url).json(&set_identity::SetIdentity { identity }).send().await;
        if res.is_err() {
            return false;
        }
        res.unwrap().status().is_success()
    }
}