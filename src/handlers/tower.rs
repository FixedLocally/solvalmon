use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};
use rocket::State;
use serde_json::json;
// use solana_core::consensus::TowerVersions;

use crate::{auth::Auth, config::Config, responder::ApiResponder};

#[rocket::get("/tower")]
pub async fn get(_auth: Auth, config: &State<Config>) -> ApiResponder {
    let tower_path = format!("{}/tower-1_9-{}.bin", config.ledger_dir, config.primary_id.to_string());
    // read tower file to string
    let tower = std::fs::read(&tower_path).unwrap();
    // let tower_struct = bincode::deserialize(&tower[0x4c..]).map(TowerVersions::V1_14_11).unwrap();
    ApiResponder::success(
        Some(json!({
            "tower": BASE64_STANDARD_NO_PAD.encode(tower),
            // "decoded": tower_struct,
        })),
        "tower".to_string(),
    )
}

#[rocket::post("/tower", data = "<tower>")]
pub async fn post(_auth: Auth, config: &State<Config>, tower: Vec<u8>) -> ApiResponder {
    let node_id = config.rpc_client.get_identity().await.unwrap();
    if node_id == config.primary_id {
        return ApiResponder::error("Refused to override my own tower".to_string());
    }
    let tower_path = format!("{}/tower-1_9-{}.bin", config.ledger_dir, config.primary_id.to_string());
    std::fs::write(&tower_path, &tower).unwrap();
    ApiResponder::success(
        Some(json!({
            "tower": BASE64_STANDARD_NO_PAD.encode(&tower),
        })),
        "tower".to_string(),
    )
}