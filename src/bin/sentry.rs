use solvalmon::sentry::{config::SentryConfig, sentry::run};

#[tokio::main]
async fn main() {
    let config = SentryConfig::new("sentry.json");
    run(config).await;
}