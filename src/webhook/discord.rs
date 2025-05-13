use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Discord {
    webhook_url: String,
}

impl Discord {
    pub fn new(webhook_url: String) -> Self {
        Self { webhook_url }
    }
}

impl super::Webhook for Discord {
    async fn send_message(&self, message: &String) -> Result<(), String> {
        let client = reqwest::Client::new();
        let res = client.post(&self.webhook_url)
            .header("Content-Type", "application/json")
            .body(format!(r#"{{"content": "{}"}}"#, message))
            .send()
            .await;

        match res {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    Err(format!("Failed to send message: {}", response.status()))
                }
            }
            Err(e) => Err(format!("Error sending message: {}", e)),
        }
    }
}