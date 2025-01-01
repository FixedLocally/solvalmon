use std::{fs, net::Ipv4Addr, vec};

use solvalmon::handlers::{error::{bad_request, internal_error, not_found, unauthorised}, post, set_identity, stats, status, tower};
use solvalmon::config;
use rocket::{catchers, config::{MutualTls, TlsConfig}, launch, Config};

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
async fn rocket() -> _ {
    panic_if_missing(TLS_CERT_PATH);
    panic_if_missing(TLS_KEY_PATH);
    panic_if_missing(MTLS_CA_PATH);
    panic_if_missing(CONFIG_PATH);
    let config = config::Config::new(CONFIG_PATH).await;
    rocket::build()
        .manage(config)
        .mount("/", rocket::routes![status::get, stats::get, tower::get, tower::post, post::post, set_identity::post])
        .register("/", catchers![not_found, internal_error, unauthorised, bad_request])
        .configure(Config {
            address: std::net::IpAddr::V4("0.0.0.0".parse::<Ipv4Addr>().unwrap()),
            port: 8888,
            tls: Some(TlsConfig::from_paths(TLS_CERT_PATH, TLS_KEY_PATH)
                    .with_mutual(MutualTls::from_path(MTLS_CA_PATH))),
            ..Config::default()
        })
}
