use rocket::catch;

#[catch(400)]
pub fn bad_request() -> &'static str {
    "bad request"
}

#[catch(401)]
pub fn unauthorised() -> &'static str {
    "unauthorised"
}

#[catch(404)]
pub fn not_found() -> &'static str {
    "not found"
}

#[catch(500)]
pub fn internal_error() -> &'static str {
    "server error, check console"
}