use serde::Deserialize;
use serde_json::json;

use crate::{auth::SignedPayload, responder::ApiResponder};

#[derive(Debug, Deserialize)]
pub struct Post {
    pub payload: String,
}

#[rocket::post("/post", data = "<payload>")]
pub async fn post(payload: SignedPayload<Post>) -> ApiResponder {
    ApiResponder::success(
        Some(json!({
            "payload": payload.inner.payload,
        })),
        "post".to_string(),
    )
}