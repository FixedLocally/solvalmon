use serde::Serialize;
use solana_client::nonblocking::rpc_client;

mod config;

#[derive(Debug, Default, Serialize)]
struct Status {
    slot: u64,
    identity: String,
    version: String,
}

#[tokio::main]
async fn main() {
    // deserialise config from config.json
    let config = config::Config::new("config.json").unwrap();
    let rpc_client = rpc_client::RpcClient::new(format!("http://127.0.0.1:{}", config.rpc_port));
    let (slot, identity, version) = tokio::join!(
        rpc_client.get_slot(),
        rpc_client.get_identity(),
        rpc_client.get_version(),
    );
    let status = Status {
        slot: slot.unwrap(),
        identity: identity.unwrap().to_string(),
        version: version.unwrap().to_string(),
    };
    println!("{:?}", status);
}
