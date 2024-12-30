use rocket::{mtls::Certificate, serde::json::Json, State};
use serde::Deserialize;
use serde_json::json;

use crate::{config::Config, responder::ApiResponder};

#[derive(Debug, Deserialize, )]
pub enum IdentityVariant {
    Primary,
    Secondary,
}

#[derive(Debug, Deserialize)]
pub struct SetIdentity {
    pub identity: IdentityVariant,
}

#[rocket::post("/set_identity", data = "<identity>")]
pub async fn post(_auth: Certificate<'_>, identity: Json<SetIdentity>, config: &State<Config>) -> ApiResponder {
    let identity_path = match identity.identity {
        IdentityVariant::Primary => &config.keys.primary,
        IdentityVariant::Secondary => &config.keys.secondary,
    };
    let output = std::process::Command::new(&config.validator_binary)
        .arg("set-identity")
        .arg(identity_path)
        .output()
        .expect("failed to execute process");
    let version = String::from_utf8_lossy(&output.stdout);
    ApiResponder::success(Some(json!({
        "version": version.to_string(),
        "identity": identity_path.to_string(),
    })), "set_identity".to_string())
}