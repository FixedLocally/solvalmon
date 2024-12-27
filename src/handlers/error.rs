use rocket::catch;


#[catch(404)]
pub fn not_found() -> &'static str {
    "not found"
}

#[catch(500)]
pub fn internal_error() -> &'static str {
    "server error, check console"
}