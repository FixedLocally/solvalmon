use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};
use rocket::State;
use serde_json::json;

use crate::{responder::ApiResponder, ConfigWrapper};


#[rocket::get("/tower")]
pub async fn handler(config: &State<ConfigWrapper>) -> ApiResponder {
    let tower_path = format!("{}/tower-1_9-{}.bin", config.config.ledger_dir, config.primary_id.to_string());
    // read tower file to string
    let tower = std::fs::read(tower_path).unwrap();
    ApiResponder::success(
        Some(json!({
            "tower": BASE64_STANDARD_NO_PAD.encode(tower),
        })),
        "tower".to_string(),
    )
}
