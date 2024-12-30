use rocket::{mtls::Certificate, State};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::join;

use crate::{responder::ApiResponder, config::Config};

#[derive(Debug, Default, Serialize, Deserialize)]
struct Status<'r> {
    slot: u64,
    identity: &'r str,
    version: &'r str,
    identity_balance: u64,
    genesis_hash: &'r str,
}

#[rocket::get("/status")]
pub async fn get(_auth: Certificate<'_>, config: &State<Config>) -> ApiResponder {
    let (slot, identity, version, acct, genesis_hash) = join!(
        config.rpc_client.get_slot(),
        config.rpc_client.get_identity(),
        config.rpc_client.get_version(),
        config.rpc_client.get_account(&config.primary_id),
        config.rpc_client.get_genesis_hash(),
    );
    ApiResponder::success(
        Some(json!(Status {
            slot: slot.unwrap(),
            identity: &identity.unwrap().to_string()[..],
            version: &version.unwrap().to_string()[..],
            identity_balance: acct.unwrap().lamports,
            genesis_hash: &genesis_hash.unwrap().to_string()[..],
        })),
        "status".to_string(),
    )
}
