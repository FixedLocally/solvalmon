use std::vec;

use config::Config;
use handlers::{error::{internal_error, not_found}, post, stats, status, tower};
use rocket::{catchers, launch};
use solana_sdk::signer::Signer;

mod auth;
mod config;
mod handlers;
mod responder;

#[launch]
fn rocket() -> _ {
    if let Ok(admin) = solana_sdk::signature::read_keypair_file("admin.json") {
        println!("GET /stats: {}", admin.sign_message(&"GET /stats".as_bytes()));
        println!("GET /status: {}", admin.sign_message(&"GET /status".as_bytes()));
        println!("GET /tower: {}", admin.sign_message(&"GET /tower".as_bytes()));
        println!("POST /tower <hash>: {}", admin.sign_message(&"POST /tower d258406eafcdaf3fbed1ea84ca25271baea80515fd6beeb963bc7a1632ab457d".as_bytes()));
    }
    let config_wrapper = Config::new("config.json");
    rocket::build()
        .manage(config_wrapper)
        .mount("/", rocket::routes![status::get, stats::get, tower::get, tower::post, post::post])
        .register("/", catchers![not_found, internal_error])
}
