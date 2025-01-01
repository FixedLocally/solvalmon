use std::{cmp::max, fmt::Display, str::FromStr, sync::Arc, thread, time::Duration};
use futures::stream::FuturesUnordered;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, vote::state::VoteState};
use crate::handlers::status::Status;

use super::{client::SentryClient, config::SentryConfig};
use tokio::join;

#[derive(Debug, Default)]
struct SanityCheckResult {
    identity_balance_low: bool,
    rpc_unhealthy: bool,
    node_unhealthy: bool,
    primary_node: String,
    node_slot: u64,
    ref_slot: u64,
    current_slot: u64,
    vote_distance: u64,
    healthy_node_count: u8,
    total_node_count: u8,

    identity_balance_low_since_ms: u128,
    delinquent_since_ms: u128,
    rpc_unhealthy_since_ms: u128,
}

impl Display for SanityCheckResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SanityCheckResult {{\n identity_balance_ok: {},\n rpc_health_ok: {},\n node_health_ok: {},\n primary_node: {},\n slot: {} / {} / {},\n vote_distance: {}, node_count: {} / {}\n",
            !self.identity_balance_low,
            !self.rpc_unhealthy,
            !self.node_unhealthy,
            self.primary_node,
            self.node_slot,
            self.ref_slot,
            self.current_slot,
            self.vote_distance,
            self.healthy_node_count,
            self.total_node_count,
        )?;
        if self.identity_balance_low {
            write!(f, " identity_balance_low_since_ms: {}\n", self.identity_balance_low_since_ms)?;
        }
        if self.node_unhealthy {
            write!(f, " delinquent_since_ms: {}\n", self.delinquent_since_ms)?;
        }
        if self.rpc_unhealthy {
            write!(f, " rpc_unhealthy_since_ms: {}\n", self.rpc_unhealthy_since_ms)?;
        }
        write!(f, "}}")
    }
}

pub async fn run(config: SentryConfig) {
    let client = Arc::new(config.http_client);
    // this should never be freed by design
    let nodes: &[SentryClient] = config.nodes.iter().map(|node| SentryClient::new(node.clone(), Arc::clone(&client))).collect::<Vec<_>>().leak();
    let rpc_client = RpcClient::new_with_commitment(config.external_rpc, CommitmentConfig::processed());
    let vote_key = Pubkey::from_str(&config.vote_id).unwrap();
    let vote = VoteState::deserialize(&rpc_client.get_account(&vote_key).await.unwrap().data).unwrap();
    let identity = vote.authorized_voters().first().unwrap().1;
    let mut sanity_check_result = SanityCheckResult::default();
    loop {
        let statuses: FuturesUnordered<_> = nodes.iter().map(|node| tokio::spawn(node.get_status())).collect();
        let (statuses, vote_account, identity_balance, ref_slot) = join!(
            futures::future::join_all(statuses),
            rpc_client.get_account(&vote_key),
            rpc_client.get_balance(identity),
            rpc_client.get_slot(),
        );
        let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
        // statuses.iter().for_each(|status| {
        //     println!("{:?}", status);
        // });
        let vote = VoteState::deserialize(&vote_account.unwrap().data).unwrap();
        let identity_balance = identity_balance.unwrap();
        let ref_slot = ref_slot.unwrap();
        // println!("{:?} {} {}", vote.votes[vote.votes.len() - 1], identity_balance, ref_slot);
        // run sanity checks
        let identity_balance_low = identity_balance < config.checks.identity_balance;
        let node_slot = statuses.iter().map(|status| status.as_ref().unwrap().slot).max().unwrap();
        let rpc_unhealthy = ref_slot < node_slot - config.checks.unhealthy_threshold;
        let node_unhealthy = node_slot < ref_slot - config.checks.unhealthy_threshold;
        let primary_node_status = statuses.iter()
            .filter(|x| x.as_ref().unwrap().identity == identity.to_string()).next();
        let primary_node_status = match primary_node_status {
            Some(status) => status.as_ref().unwrap(),
            None => {
                &Status::unreachable()
            }
        };
        let current_slot = max(ref_slot, node_slot);
        let vote_distance = current_slot - vote.votes[vote.votes.len() - 1].lockout.slot();
        let healthy_node_count = statuses.iter().filter(|status| status.as_ref().unwrap().slot >= current_slot - config.checks.unhealthy_threshold).count() as u8;
        let total_node_count = statuses.len() as u8;
        sanity_check_result = SanityCheckResult {
            identity_balance_low,
            rpc_unhealthy,
            node_unhealthy,
            primary_node: primary_node_status.hostname.clone(),
            node_slot,
            ref_slot,
            current_slot,
            vote_distance,
            healthy_node_count,
            total_node_count,

            identity_balance_low_since_ms: if identity_balance_low { if sanity_check_result.identity_balance_low_since_ms == 0 {now_ms} else {sanity_check_result.identity_balance_low_since_ms} } else { 0 },
            delinquent_since_ms: if node_unhealthy { if sanity_check_result.delinquent_since_ms == 0 {now_ms} else {sanity_check_result.delinquent_since_ms} } else { 0 },
            rpc_unhealthy_since_ms: if rpc_unhealthy { if sanity_check_result.rpc_unhealthy_since_ms == 0 {now_ms} else {sanity_check_result.rpc_unhealthy_since_ms} } else { 0 },
        };
        println!("{}", sanity_check_result);
        thread::sleep(Duration::from_secs(2));
    }
}