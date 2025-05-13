use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Telegram {
    token: String,
    chat_id: String,
}

impl Telegram {
    pub fn new(token: String, chat_id: String) -> Self {
        Self { token, chat_id }
    }
}

impl super::Webhook for Telegram {
    async fn send_message(&self, message: &String) -> Result<(), String> {
        let client = reqwest::Client::new();
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);
        let params = [("chat_id", &self.chat_id), ("text", &message.to_string())];

        let res = client.post(&url)
            .form(&params)
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