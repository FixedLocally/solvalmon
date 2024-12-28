use rocket::{http::ContentType, response::Responder};
use serde_json::{json, Value};


pub struct ApiResponder {
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

    pub fn success_empty() -> Self {
        Self::new(true, None, None, "".to_string())
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
