use rocket::{http::Status, request::{FromRequest, Outcome}, Request, State};

use crate::{auth::lib::check_sig, config::Config};

pub struct Auth {}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let config = State::<Config>::get(req.rocket()).unwrap();
        let str_to_sign = format!("{} {}", req.method(), req.uri().path().as_str());
        
        match req.headers().get_one("x-api-key") {
            None => Outcome::Error((Status::Unauthorized, ())),
            Some(key) if check_sig(config.admin, key, &str_to_sign) => Outcome::Success(Auth {}),
            Some(_) => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}