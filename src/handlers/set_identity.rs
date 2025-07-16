use std::{io::Read, thread, time::Duration};

use rocket::{mtls::Certificate, serde::json::Json, State};
use serde::{Deserialize, Serialize};

use crate::{monitor::config::ValidatorConfig, responder::ApiResponder};

#[derive(Debug, Deserialize, Serialize)]
pub enum IdentityVariant {
    Primary,
    Secondary,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SetIdentity {
    pub identity: IdentityVariant,
}

#[rocket::post("/set_identity", data = "<identity>")]
pub async fn post(_auth: Certificate<'_>, identity: Json<SetIdentity>, config: &State<ValidatorConfig>) -> ApiResponder {
    let identity_path = match identity.identity {
        IdentityVariant::Primary => &config.keys.primary,
        IdentityVariant::Secondary => &config.keys.secondary,
    };
    if let Some(firedancer) = &config.firedancer {
        // run `firedancer.fdctl "set-identity" identity_path "--config" config.firedancer.config_path --force`
        let child = std::process::Command::new(&firedancer.fdctl)
            .arg("set-identity")
            .arg(identity_path)
            .arg("--config")
            .arg(&firedancer.config_path)
            .arg("--force")
            .spawn();
        if child.is_err() {
            return ApiResponder::error("cannot spawn child process".to_string());
        }
        let mut child = child.unwrap();
        // ...and give it 15s to complete (normally it should complete within 1-2s)
        thread::sleep(Duration::from_secs(15));

        // if it completes within the time limit, handle the result
        if let Ok(Some(status)) = child.try_wait() {
            if status.success() {
                return ApiResponder::success_empty();
            } else {
                let err_msg = if let Some(mut stderr) = child.stderr {
                    let mut buf = [0u8; 1024];
                    if let Ok(size) = stderr.read(&mut buf) {
                        String::from_utf8_lossy(&buf[0..size]).to_string()
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                };
                return ApiResponder::error(format!("Failed to execute fdctl command: {} ({})", err_msg, status.code().unwrap_or(-1)));
            }
        }

        // otherwise consider it a failure and kill the process (as well as the validator)
        match child.kill() {
            Ok(_) => {
                // for now we find the fdctl process with the largest memory usage (assuming 1 validator per server)
                // and murder it to have it restart in a clean state
                let _ = child.wait(); // reap the child
                if let Ok(output) = std::process::Command::new("ps")
                    .arg("aux")
                    .output()
                {
                    if let Ok(ps_output) = String::from_utf8(output.stdout) {
                        let mut max_memory = 0u64;
                        let mut max_memory_pid = 0u64;
                        for line in ps_output.lines().skip(1) {
                            let columns: Vec<&str> = line.split_whitespace().collect();
                            if columns.len() > 10 && columns[10].contains("fdctl") {
                                if let Ok(mem) = columns[5].parse::<u64>() {
                                    if mem > max_memory {
                                        max_memory = mem;
                                        max_memory_pid = columns[1].parse::<u64>().unwrap();
                                    }
                                }
                            }
                        }
                        if max_memory_pid != 0 {
                            println!("fdctl process using the most memory - killing: {}", max_memory_pid);
                            let _ = std::process::Command::new("kill")
                                .arg("-9")
                                .arg(max_memory_pid.to_string())
                                .spawn()
                                .and_then(|mut child| child.wait());
                        }
                    }
                }

                return ApiResponder::error(format!("Process timed out and killed"));
            },
            Err(e) => return ApiResponder::error(format!("Process timed out and can't be killed {e}"))
        }
    } else {
        match config.admin_client().await.set_identity(identity_path.clone(), false).await {
            Ok(_) => return ApiResponder::success_empty(),
            Err(e) => return ApiResponder::error(e.to_string()),
        }
    }
    
}