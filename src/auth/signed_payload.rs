use rocket::{data::{FromData, Outcome, ToByteUnit}, http::Status, Data, Request, State};
use serde::Deserialize;
use sha256::digest;

use crate::config::Config;

use super::lib::check_sig;

pub struct SignedPayload<T: for<'a> Deserialize<'a>> {
    pub inner: T,
}

#[rocket::async_trait]
impl <'r, T: for<'a> Deserialize<'a>> FromData<'r> for SignedPayload<T> {
    type Error = ();

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let config = State::<Config>::get(req.rocket()).unwrap();
        let mut payload = String::new();

        // read up to 32KiB of data into payload
        match data.open(32.kibibytes()).into_string().await {
            Ok(s) => payload.push_str(&s),
            Err(_) => return Outcome::Error((Status::InternalServerError, ())),
        }
        let str_to_sign = format!("{} {} {}", req.method(), req.uri().path().as_str(), digest(payload.as_bytes()));
        
        match req.headers().get_one("x-api-key") {
            None => Outcome::Error((Status::Unauthorized, ())),
            Some(key) if check_sig(config.admin, key, &str_to_sign) => Outcome::Success(SignedPayload {
                inner: serde_json::from_str(&payload).unwrap(),
            }),
            Some(_) => Outcome::Error((Status::Unauthorized, ())),
        }

        
    }
}