use rocket::{mtls::Certificate, serde::json::Json, State};
use serde::{Deserialize, Serialize};

use crate::{config::ValidatorConfig, responder::ApiResponder};

#[derive(Debug, Deserialize, Serialize)]
pub enum IdentityVariant {
    Primary,
    Secondary,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SetIdentity {
    pub identity: IdentityVariant,
}

#[rocket::post("/set_identity", data = "<identity>")]
pub async fn post(_auth: Certificate<'_>, identity: Json<SetIdentity>, config: &State<ValidatorConfig>) -> ApiResponder {
    let identity_path = match identity.identity {
        IdentityVariant::Primary => &config.keys.primary,
        IdentityVariant::Secondary => &config.keys.secondary,
    };
    match config.admin_client().await.set_identity(identity_path.clone(), false).await {
        Ok(_) => return ApiResponder::success_empty(),
        Err(e) => return ApiResponder::error(e.to_string()),
    }
    
}