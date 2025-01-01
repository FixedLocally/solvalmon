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
    match config.admin_client.set_identity(identity_path.clone(), true).await {
        Ok(_) => return ApiResponder::success(Some(json!({
            "identity": identity_path.to_string(),
        })), "set_identity".to_string()),
        Err(e) => return ApiResponder::error(e.to_string()),
    }
    
}