use std::{path::Path, str::FromStr};

use agave_validator::admin_rpc_service;
use serde::Deserialize;
use solana_client::nonblocking::rpc_client;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signer::Signer};

#[derive(Deserialize, Debug)]
struct ValidatorConfigInner {
    pub vote_account: String,
    pub ledger_dir: String,
    pub keys: KeysConfig,
}

impl ValidatorConfigInner {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }
}

#[derive(Deserialize, Debug)]
pub struct KeysConfig {
    pub primary: String,
    pub secondary: String,
}

pub struct ValidatorConfig {
    pub rpc_client: rpc_client::RpcClient,
    pub primary_id: Pubkey,
    pub vote_id: Pubkey,
    pub ledger_dir: String,
    pub keys: KeysConfig,
}

impl ValidatorConfig {
    pub async fn new(path: &str) -> Self {
        let config = ValidatorConfigInner::new(path).unwrap();
        let admin_client = match admin_rpc_service::connect(&Path::new(&config.ledger_dir)).await {
            Ok(client) => client,
            Err(e) => panic!("Failed to connect to admin RPC: {}", e),
        };
        let rpc_addr = match admin_client.rpc_addr().await.unwrap() {
            Some(addr) => addr,
            None => panic!("Failed to get RPC address"),
        };
        let rpc_client = rpc_client::RpcClient::new_with_commitment(format!("http://{}", rpc_addr), CommitmentConfig::processed());
        let primary_keypair = solana_sdk::signature::read_keypair_file(&config.keys.primary).unwrap();
        let primary_id = primary_keypair.pubkey();
        Self {
            rpc_client,
            primary_id,
            vote_id: Pubkey::from_str(&config.vote_account).unwrap(),
            ledger_dir: config.ledger_dir,
            keys: config.keys,
        }
    }

    pub async fn admin_client(&self) -> admin_rpc_service::AdminRpcClient {
        match admin_rpc_service::connect(&Path::new(&self.ledger_dir)).await {
            Ok(client) => client,
            Err(e) => panic!("Failed to connect to admin RPC: {}", e),
        }
    }
}
