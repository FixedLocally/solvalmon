pub mod discord;
pub mod telegram;

use crate::sentry::sentry::SanityCheckResult;

pub trait Webhook {
    async fn send_message(&self, sanity_check: Option<&SanityCheckResult>, message: &String) -> Result<(), String>;
}
