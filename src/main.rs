use std::{fs, net::Ipv4Addr, vec};

use handlers::{error::{internal_error, not_found, unauthorised}, post, set_identity, stats, status, tower};
use rocket::{catchers, config::{MutualTls, TlsConfig}, launch, Config};
use solana_sdk::signer::Signer;

mod config;
mod handlers;
mod responder;

const TLS_CERT_PATH: &str = "pki/tls.crt";
const TLS_KEY_PATH: &str = "pki/tls.key";
const MTLS_CA_PATH: &str = "pki/mtls_ca.crt";
const CONFIG_PATH: &str = "config.json";

fn panic_if_missing(path: &str) {
    match fs::exists(path) {
        Ok(true) => {},
        _ => panic!("{} not found", path),
    };
}

#[launch]
fn rocket() -> _ {
    if let Ok(admin) = solana_sdk::signature::read_keypair_file("admin.json") {
        println!("GET /stats: {}", admin.sign_message(&"GET /stats".as_bytes()));
        println!("GET /status: {}", admin.sign_message(&"GET /status".as_bytes()));
        println!("GET /tower: {}", admin.sign_message(&"GET /tower".as_bytes()));
        println!("POST /tower <hash>: {}", admin.sign_message(&"POST /tower d258406eafcdaf3fbed1ea84ca25271baea80515fd6beeb963bc7a1632ab457d".as_bytes()));
    }
    panic_if_missing(TLS_CERT_PATH);
    panic_if_missing(TLS_KEY_PATH);
    panic_if_missing(MTLS_CA_PATH);
    panic_if_missing(CONFIG_PATH);
    let config_wrapper = config::Config::new(CONFIG_PATH);
    rocket::build()
        .manage(config_wrapper)
        .mount("/", rocket::routes![status::get, stats::get, tower::get, tower::post, post::post, set_identity::post])
        .register("/", catchers![not_found, internal_error, unauthorised])
        .configure(Config {
            address: std::net::IpAddr::V4("0.0.0.0".parse::<Ipv4Addr>().unwrap()),
            port: 8888,
            tls: Some(TlsConfig::from_paths(TLS_CERT_PATH, TLS_KEY_PATH)
                    .with_mutual(MutualTls::from_path(MTLS_CA_PATH))),
            ..Config::default()
        })
}
