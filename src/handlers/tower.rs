use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};
use rocket::{mtls::Certificate, serde::json::Json, State};
use serde::Deserialize;
use serde_json::json;
// use solana_core::consensus::TowerVersions;

use crate::{config::Config, responder::ApiResponder};

#[derive(Debug, Deserialize)]
pub struct PostTower {
    tower: String,
}

#[rocket::get("/tower")]
pub async fn get(_auth: Certificate<'_>, config: &State<Config>) -> ApiResponder {
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
pub async fn post(_auth: Certificate<'_>, config: &State<Config>, tower: Json<PostTower>) -> ApiResponder {
    let node_id = config.rpc_client.get_identity().await.unwrap();
    if node_id == config.primary_id {
        return ApiResponder::error("Refused to override my own tower".to_string());
    }
    let tower_path = format!("{}/tower-1_9-{}.bin", config.ledger_dir, config.primary_id.to_string());
    let raw_tower = BASE64_STANDARD_NO_PAD.decode(&tower.tower);
    if raw_tower.is_err() {
        return ApiResponder::error("Invalid base64".to_string());
    }
    std::fs::write(&tower_path, raw_tower.unwrap()).unwrap();
    ApiResponder::success_empty()
}