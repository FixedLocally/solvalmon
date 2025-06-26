use rocket::{mtls::Certificate, serde::json::Json, State};
use serde::{Deserialize, Serialize};

use crate::{monitor::config::ValidatorConfig, responder::ApiResponder};

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
    if let Some(firedancer) = &config.firedancer {
        // run `firedancer.fdctl "set-identity" identity_path "--config" config.firedancer.config_path --force`
        let output = std::process::Command::new(&firedancer.fdctl)
            .arg("set-identity")
            .arg(identity_path)
            .arg("--config")
            .arg(&firedancer.config_path)
            .arg("--force")
            .output();
        match output {
            Ok(output) => {
                if output.status.success() {
                    return ApiResponder::success_empty();
                } else {
                    let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
                    return ApiResponder::error(err_msg);
                }
            }
            Err(e) => return ApiResponder::error(format!("Failed to execute fdctl command: {}", e)),
        }
    } else {
        match config.admin_client().await.set_identity(identity_path.clone(), false).await {
            Ok(_) => return ApiResponder::success_empty(),
            Err(e) => return ApiResponder::error(e.to_string()),
        }
    }
    
}