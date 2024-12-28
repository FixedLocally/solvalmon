use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};
use rocket::State;
use serde_json::json;

use crate::{auth::Auth, config::Config, responder::ApiResponder};


#[rocket::get("/tower")]
pub async fn handler(_auth: Auth, config: &State<Config>) -> ApiResponder {
    let tower_path = format!("{}/tower-1_9-{}.bin", config.ledger_dir, config.primary_id.to_string());
    // read tower file to string
    let tower = std::fs::read(tower_path).unwrap();
    ApiResponder::success(
        Some(json!({
            "tower": BASE64_STANDARD_NO_PAD.encode(tower),
        })),
        "tower".to_string(),
    )
}
