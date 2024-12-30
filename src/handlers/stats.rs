use rocket::{mtls::Certificate, State};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_client::rpc_response::RpcVoteAccountInfo;

use crate::{responder::ApiResponder, config::Config};


#[derive(Debug, Default, Serialize, Deserialize)]
struct Stats {
    my_credits: u64,
    median_credits: u64,
}

fn get_current_credits(vote_account: &RpcVoteAccountInfo) -> u64 {
    let i = vote_account.epoch_credits.iter().enumerate().fold(0, |acc, (i, (epoch, _, _))| {
        if epoch > &vote_account.epoch_credits[acc].0 {
            i
        } else {
            acc
        }
    });
    vote_account.epoch_credits[i].1 - vote_account.epoch_credits[i].2
}

#[rocket::get("/stats")]
pub async fn get(_auth: Certificate<'_>, config: &State<Config>) -> ApiResponder {
    let mut cluster_credits = vec![];
    let mut my_credits = 0;
    config.rpc_client.get_vote_accounts().await.map_or_else(
        |e| ApiResponder::error(e.to_string()),
        |vote_accounts| {
            for vote_account in vote_accounts.current {
                if vote_account.vote_pubkey == config.vote_id.to_string() {
                    my_credits = get_current_credits(&vote_account);
                }
                cluster_credits.push(get_current_credits(&vote_account));
            }
            ApiResponder::success(None, "stats".to_string())
        },
    );
    cluster_credits.sort();
    let len = cluster_credits.len();
    let median_credits = match len % 2 {
        0 => (cluster_credits[len / 2 - 1] + cluster_credits[len / 2]) / 2,
        1 => cluster_credits[len / 2],
        _ => unreachable!(),
    };
    ApiResponder::success(
        Some(json!(Stats {
            my_credits,
            median_credits,
        })),
        "stats".to_string(),
    )
}