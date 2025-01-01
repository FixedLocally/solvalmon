use rocket::{mtls::Certificate, State};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::join;

use crate::{responder::ApiResponder, config::ValidatorConfig};

#[derive(Debug, Default, Serialize, Deserialize)]
struct Status<'r> {
    slot: u64,
    identity: &'r str,
    version: &'r str,
    identity_balance: u64,
    genesis_hash: &'r str,
    uptime_ms: u64,
}

#[rocket::get("/status")]
pub async fn get(_auth: Certificate<'_>, config: &State<ValidatorConfig>) -> ApiResponder {
    let (slot, identity, version, acct, genesis_hash, start_time) = join!(
        config.rpc_client.get_slot(),
        config.rpc_client.get_identity(),
        config.rpc_client.get_version(),
        config.rpc_client.get_account(&config.primary_id),
        config.rpc_client.get_genesis_hash(),
        config.admin_client.start_time(),
    );
    let uptime_ms = start_time.unwrap().elapsed().unwrap().as_millis() as u64;
    ApiResponder::success(
        Some(json!(Status {
            slot: slot.unwrap(),
            identity: &identity.unwrap().to_string()[..],
            version: &version.unwrap().to_string()[..],
            identity_balance: acct.unwrap().lamports,
            genesis_hash: &genesis_hash.unwrap().to_string()[..],
            uptime_ms,
        })),
        "status".to_string(),
    )
}
