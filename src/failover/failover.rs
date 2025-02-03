use std::sync::Arc;

use futures::stream::FuturesUnordered;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, vote::state::VoteState};
use tokio::join;

use crate::{handlers::set_identity::IdentityVariant, sentry::{client::SentryClient, config::SentryConfig}};

pub async fn run(config: SentryConfig, new_host: &str) {
    let rpc_client = RpcClient::new_with_commitment(config.external_rpc, CommitmentConfig::processed());
    let client = Arc::new(config.http_client);
    let nodes = config.nodes.iter().map(|node| SentryClient::new(node.clone(), Arc::clone(&client))).collect::<Vec<_>>().leak();
    let statuses: FuturesUnordered<_> = nodes.iter().map(|node| tokio::spawn(node.get_status())).collect();
    let (statuses,) = join!(
        futures::future::join_all(statuses),
    );
    let vote = VoteState::deserialize(&rpc_client.get_account(&config.vote_id.parse().unwrap()).await.unwrap().data).unwrap();
    let identity = vote.authorized_voters().first().unwrap().1;
    let current_primary = statuses.iter().find(|x| x.as_ref().unwrap().identity == identity.to_string());
    let sentry_client_2 = SentryClient::new(new_host.to_string(), Arc::clone(&client));
    if let Some(current_primary) = current_primary {
        if let Ok(current_primary) = current_primary {
            if current_primary.hostname == new_host {
                println!("Primary is already set to {}", new_host);
                return;
            }
            let sentry_client = SentryClient::new(current_primary.hostname.clone(), Arc::clone(&client));
            println!("Getting tower from primary...");
            let tower = sentry_client.get_tower().await.unwrap();
            println!("Demoting current primary...");
            sentry_client.set_identity(IdentityVariant::Secondary).await;
            println!("Posting tower to new primary...");
            sentry_client_2.post_tower(tower.tower).await;
            println!("Promoting new primary...");
            sentry_client_2.set_identity(IdentityVariant::Primary).await;
            println!("Done!");
        } else {
            unreachable!();
        }
    } else {
        println!("Primary not found!");
    }
}