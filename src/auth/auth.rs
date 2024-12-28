use rocket::{http::Status, request::{FromRequest, Outcome}, Request, State};
use solana_sdk::signature::Signature;

use crate::config::Config;

pub struct Auth {}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let config = State::<Config>::get(req.rocket()).unwrap();
        let payload = format!("{} {}", req.method(), req.uri().path().as_str());
        
        print!("{}: ", payload);
        let is_valid = |key: &str| -> bool {
            key.parse::<Signature>().map_or_else(|_| false, |sig| {
                sig.verify(&config.admin.to_bytes(), payload.as_bytes())
            })
        };
        
        match req.headers().get_one("x-api-key") {
            None => Outcome::Error((Status::Unauthorized, ())),
            Some(key) if is_valid(key) => Outcome::Success(Auth {}),
            Some(_) => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}