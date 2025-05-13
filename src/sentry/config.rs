use std::sync::Arc;

use chrono::Utc;
use serde::Deserialize;

use crate::webhook::{discord::Discord, telegram::Telegram, Webhook as _};

#[derive(Deserialize, Debug)]
pub struct PkiConfig {
    pub cert: String,
    pub key: String,
}

#[derive(Deserialize, Debug)]
pub struct SentryChecks {
    pub identity_balance: u64,
    pub unhealthy_threshold: u64,
}

#[derive(Deserialize, Debug)]
pub struct SentryConfigInner {
    pub nodes: Vec<String>,
    pub auth: PkiConfig,
    pub external_rpc: String,
    pub vote_id: String,
    pub checks: SentryChecks,
    pub discord: Option<Discord>,
    pub telegram: Option<Telegram>,
}

impl SentryConfigInner {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }
}
pub struct SentryConfig {
    pub nodes: Vec<String>,
    pub http_client: Arc<reqwest::Client>,
    pub external_rpc: String,
    pub vote_id: String,
    pub checks: SentryChecks,
    pub discord: Option<Discord>,
    pub telegram: Option<Telegram>,
}

impl SentryConfig {
    pub fn new(path: &str) -> Self {
        let inner = SentryConfigInner::new(path).unwrap();
        // create https clients for each node
        let identity = reqwest::Identity::from_pkcs8_pem(
            &std::fs::read(&inner.auth.cert).unwrap()[..],
            &std::fs::read(&inner.auth.key).unwrap()[..],
        ).expect("unable to create identity");
        let client = reqwest::Client::builder()
            .identity(identity)
            .build()
            .unwrap();
        Self {
            nodes: inner.nodes,
            http_client: Arc::new(client),
            external_rpc: inner.external_rpc,
            vote_id: inner.vote_id,
            checks: inner.checks,
            discord: inner.discord,
            telegram: inner.telegram,
        }
    }

    pub async fn send_webhook(&self, message: &String) {

        let dt = Utc::now();
        let timestamp: i64 = dt.timestamp();
        println!("[{}] {}", timestamp, message);
        if let Some(discord) = &self.discord {
            discord.send_message(message).await.unwrap();
        }
        if let Some(telegram) = &self.telegram {
            telegram.send_message(message).await.unwrap();
        }
    }
}