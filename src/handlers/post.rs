use rocket::{mtls::Certificate, serde::json::Json};
use serde::Deserialize;
use serde_json::json;

use crate::responder::ApiResponder;

#[derive(Deserialize)]
pub struct Post {
    pub payload: String,
}

#[rocket::post("/post", data = "<payload>")]
pub async fn post(_auth: Certificate<'_>, payload: Json<Post>) -> ApiResponder {
    ApiResponder::success(
        Some(json!({
            "payload": payload.payload,
        })),
        "post".to_string(),
    )
}