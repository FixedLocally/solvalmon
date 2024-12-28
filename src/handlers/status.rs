use rocket::State;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::join;

use crate::{auth::Auth, responder::ApiResponder, config::Config};

#[derive(Debug, Default, Serialize, Deserialize)]
struct Status<'r> {
    slot: u64,
    identity: &'r str,
    version: &'r str,
    identity_balance: u64,
}

#[rocket::get("/status")]
pub async fn get(_auth: Auth, config: &State<Config>) -> ApiResponder {
    let (slot, identity, version, acct) = join!(
        config.rpc_client.get_slot(),
        config.rpc_client.get_identity(),
        config.rpc_client.get_version(),
        config.rpc_client.get_account(&config.primary_id),
    );
    ApiResponder::success(
        Some(json!(Status {
            slot: slot.unwrap(),
            identity: &identity.unwrap().to_string()[..],
            version: &version.unwrap().to_string()[..],
            identity_balance: acct.unwrap().lamports,
        })),
        "status".to_string(),
    )
}
