use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};
use rocket::{catch, catchers, http::ContentType, launch, response::Responder, State};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use solana_client::nonblocking::rpc_client;
use solana_sdk::{commitment_config::CommitmentConfig, signer::Signer};

mod config;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Status<'r> {
    slot: u64,
    identity: &'r str,
    version: &'r str,
    identity_balance: u64,
}

struct ApiResponder {
    pub success: bool,
    pub message: Option<String>,
    pub inner: Option<Value>,
    pub field_name: String,
}

impl ApiResponder {
    pub fn new(success: bool, message: Option<String>, inner: Option<Value>, field_name: String) -> Self {
        Self {
            success,
            message,
            inner,
            field_name,
        }
    }

    pub fn success(inner: Option<Value>, field_name: String) -> Self {
        Self::new(true, None, inner, field_name)
    }

    pub fn error(message: String) -> Self {
        Self::new(false, Some(message), None, "error".to_string())
    }
}

impl <'r, 'o: 'r> Responder<'r, 'o> for ApiResponder {
    fn respond_to(self, _: &'r rocket::Request) -> rocket::response::Result<'o> {
        let mut json = json!({
            "success": self.success,
        });
        if let Some(message) = self.message {
            json["message"] = json!(message);
        }
        if let Some(inner) = self.inner {
            json[self.field_name] = inner;
        }
        let resp = json.to_string();
        rocket::Response::build()
            .header(ContentType::JSON)
            .sized_body(resp.len(), std::io::Cursor::new(resp))
            .ok()
    }
}

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
    rocket::build().manage(config_wrapper).mount("/", rocket::routes![status, tower]).register("/", catchers![not_found])
}

#[catch(404)]
fn not_found() -> &'static str {
    "Not found!"
}

#[rocket::get("/status")]
async fn status(config: &State<ConfigWrapper>) -> ApiResponder {
    let (slot, identity, version, acct) = tokio::join!(
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

#[rocket::get("/tower")]
async fn tower(config: &State<ConfigWrapper>) -> ApiResponder {
    let tower_path = format!("{}/tower-1_9-{}.bin", config.config.ledger_dir, config.primary_id.to_string());
    // read tower file to string
    let tower = std::fs::read(tower_path).unwrap();
    ApiResponder::success(
        Some(json!({
            "tower": BASE64_STANDARD_NO_PAD.encode(tower),
        })),
        "tower".to_string(),
    )
}