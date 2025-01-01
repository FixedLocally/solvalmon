use serde::Deserialize;

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
    pub http_client: reqwest::Client,
    pub external_rpc: String,
    pub vote_id: String,
    pub checks: SentryChecks,
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
            http_client: client,
            external_rpc: inner.external_rpc,
            vote_id: inner.vote_id,
            checks: inner.checks,
        }
    }
}