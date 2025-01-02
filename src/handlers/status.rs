use rocket::{mtls::Certificate, State};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::join;

use crate::{responder::ApiResponder, config::ValidatorConfig};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Status {
    pub hostname: String,
    pub slot: u64,
    pub identity: String,
    pub version: String,
    pub identity_balance: u64,
    pub genesis_hash: String,
    pub uptime_ms: u64,
}

impl Status {
    pub fn unreachable() -> Self {
        Self {
            hostname: hostname::get().expect("gethostname").into_string().unwrap(),
            slot: 0,
            identity: "(unreachable)".to_string(),
            version: "".to_string(),
            identity_balance: 0,
            genesis_hash: "".to_string(),
            uptime_ms: 0,
        }
    }

    pub fn not_running() -> Self {
        Self {
            hostname: hostname::get().expect("gethostname").into_string().unwrap(),
            slot: 0,
            identity: "(not running)".to_string(),
            version: "".to_string(),
            identity_balance: 0,
            genesis_hash: "".to_string(),
            uptime_ms: 0,
        }
    }
}

#[rocket::get("/status")]
pub async fn get(_auth: Certificate<'_>, config: &State<ValidatorConfig>) -> ApiResponder {
    // according to the validator's source code, none of these requires --full-rpc-api
    let (slot, identity, version, acct, genesis_hash, start_time) = join!(
        config.rpc_client.get_slot(),
        config.rpc_client.get_identity(),
        config.rpc_client.get_version(),
        config.rpc_client.get_balance(&config.primary_id),
        config.rpc_client.get_genesis_hash(),
        config.admin_client().await.start_time(),
    );
    let uptime_ms = start_time.unwrap().elapsed().unwrap().as_millis() as u64;
    if slot.is_err() || identity.is_err() || version.is_err() || acct.is_err() || genesis_hash.is_err() {
        return ApiResponder::success(
            Some(json!(Status::unreachable())),
            "status".to_string(),
        );
    }
    ApiResponder::success(
        Some(json!(Status {
            hostname: hostname::get().expect("gethostname").into_string().unwrap(),
            slot: slot.unwrap(),
            identity: identity.unwrap().to_string(),
            version: version.unwrap().to_string(),
            identity_balance: acct.unwrap(),
            genesis_hash: genesis_hash.unwrap().to_string(),
            uptime_ms,
        })),
        "status".to_string(),
    )
}
