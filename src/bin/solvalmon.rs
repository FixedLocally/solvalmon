use std::{fs, net::Ipv4Addr, vec};

use agave_validator::admin_rpc_service;
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

async fn netmon(ledger_path: String, secondary_identity_path: String) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(15));
    let mut fail_count = 0;
    loop {
        interval.tick().await;
        if !reqwest::get("https://www.google.com").await.is_ok() {
            eprintln!("No internet connectivity");
            fail_count += 1;
        } else {
            fail_count = 0;
        }
        if fail_count >= 3 {
            // set identity to secondary so we can failover
            let admin_client = admin_rpc_service::connect(&std::path::Path::new(&ledger_path)).await.unwrap();
            if let Err(e) = admin_client.set_identity(secondary_identity_path.clone(), false).await {
                eprintln!("Failed to set identity to secondary: {}", e);
            } else {
                eprintln!("Set identity to secondary due to internet connectivity failure");
            }
        }
    }
}

#[launch]
async fn rocket() -> _ {
    panic_if_missing(TLS_CERT_PATH);
    panic_if_missing(TLS_KEY_PATH);
    panic_if_missing(MTLS_CA_PATH);
    panic_if_missing(CONFIG_PATH);
    let config = config::ValidatorConfig::new(CONFIG_PATH).await;
    // monitor internet connectivity
    tokio::spawn(netmon(config.ledger_dir.clone(), config.keys.secondary.clone()));
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
