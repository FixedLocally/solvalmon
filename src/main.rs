use std::vec;

use handlers::{error::{internal_error, not_found}, stats, status, tower};
use rocket::{catch, catchers, launch};
use solana_client::nonblocking::rpc_client;
use solana_sdk::{commitment_config::CommitmentConfig, signer::Signer};

mod config;
mod handlers;
mod responder;

struct ConfigWrapper {
    config: config::Config,
    rpc_client: rpc_client::RpcClient,
    primary_id: solana_sdk::pubkey::Pubkey,
}

#[launch]
fn rocket() -> _ {
    let config = config::Config::new("config.json").unwrap();
    let rpc_client = rpc_client::RpcClient::new_with_commitment(format!("http://127.0.0.1:{}", config.rpc_port), CommitmentConfig::processed());
    let primary_keypair = solana_sdk::signature::read_keypair_file(&config.keys.primary).unwrap();
    let primary_id = primary_keypair.pubkey();
    let config_wrapper = ConfigWrapper {
        config,
        rpc_client,
        primary_id,
    };
    rocket::build().manage(config_wrapper).mount("/", rocket::routes![status::handler, stats::handler, tower::handler]).register("/", catchers![not_found, internal_error])
}
