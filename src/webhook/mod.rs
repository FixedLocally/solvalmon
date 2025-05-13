pub mod discord;
pub mod telegram;

pub trait Webhook {
    async fn send_message(&self, message: &String) -> Result<(), String>;
}
