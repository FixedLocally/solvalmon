use std::{str::FromStr, sync::Arc};

use futures::stream::FuturesUnordered;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey};
use solana_vote_program::vote_state::VoteStateV4;
use tabled::{Table, Tabled};
use tokio::join;

use crate::{handlers::status::Status, sentry::{client::SentryClient, config::SentryConfig}};

fn format_ms(ms: u64) -> String {
    let seconds = ms / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    format!("{:02}:{:02}:{:02}", hours, minutes % 60, seconds % 60)
}

impl Tabled for Status {
    const LENGTH: usize = 5;

    fn fields(&self) -> Vec<std::borrow::Cow<'_, str>> {
        vec![
            self.hostname.as_str().into(),
            self.slot.to_string().into(),
            self.identity.as_str().into(),
            self.version.as_str().into(),
            format_ms(self.uptime_ms).into(),
        ]
    }

    fn headers() -> Vec<std::borrow::Cow<'static, str>> {
        vec![
            "Hostname".into(),
            "Slot".into(),
            "Identity".into(),
            "Version".into(),
            "Uptime".into(),
        ]
    }
}

pub async fn run(config: SentryConfig) {
    let client = Arc::new(config.http_client);
    let rpc_client = RpcClient::new_with_commitment(config.external_rpc, CommitmentConfig::processed());
    let vote_key = Pubkey::from_str(&config.vote_id).unwrap();
    let vote = VoteStateV4::deserialize(&rpc_client.get_account(&vote_key).await.unwrap().data, &vote_key).unwrap();
    let identity = vote.authorized_voters.first().unwrap().1;

    let nodes = config.nodes.iter().map(|node| SentryClient::new(node.clone(), Arc::clone(&client))).collect::<Vec<_>>().leak();
    let statuses: FuturesUnordered<_> = nodes.iter().map(|node| tokio::spawn(node.get_status())).collect();
    let (statuses, vote_account, identity_balance, ref_slot) = join!(
        futures::future::join_all(statuses),
        rpc_client.get_account(&vote_key),
        rpc_client.get_balance(identity),
        rpc_client.get_slot(),
    );
    let vote = VoteStateV4::deserialize(&vote_account.unwrap().data, &vote_key).unwrap();
    let identity_balance = identity_balance.unwrap();
    let ref_slot = ref_slot.unwrap();
    println!("{}", Table::new(statuses.iter().map(|x| x.as_ref().unwrap())).to_string());
    println!("Ref slot: {}", ref_slot);
    println!("Identity balance: {}.{:09}", identity_balance / LAMPORTS_PER_SOL, identity_balance % LAMPORTS_PER_SOL);
    println!("Vote diff: {}", ref_slot.saturating_sub(vote.votes[vote.votes.len() - 1].lockout.slot()));
}