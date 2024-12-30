use std::str::FromStr;

use serde::Deserialize;
use solana_client::nonblocking::rpc_client;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signer::Signer};

#[derive(Deserialize, Debug)]
pub struct ConfigInner {
    pub rpc_port: u16,
    pub vote_account: String,
    pub ledger_dir: String,
    pub keys: KeysConfig,
    pub admin: String,
}

impl ConfigInner {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct KeysConfig {
    pub primary: String,
    pub secondary: String,
}

pub struct Config {
    pub rpc_client: rpc_client::RpcClient,
    pub primary_id: Pubkey,
    pub vote_id: Pubkey,
    pub ledger_dir: String,
    pub keys: KeysConfig,
}

impl Config {
    pub fn new(path: &str) -> Self {
        let config = ConfigInner::new(path).unwrap();
        let rpc_client = rpc_client::RpcClient::new_with_commitment(format!("http://127.0.0.1:{}", config.rpc_port), CommitmentConfig::processed());
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
}