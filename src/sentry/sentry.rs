use std::{cmp::max, fmt::Display, str::FromStr, sync::Arc, thread, time::Duration};
use futures::stream::FuturesUnordered;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, vote::state::VoteState};
use crate::handlers::{set_identity::IdentityVariant, status::Status};

use super::{client::SentryClient, config::SentryConfig};
use tokio::join;

const INFO_EMOJI: &str = "ℹ️";
const WARNING_EMOJI: &str = "⚠️";
const ERROR_EMOJI: &str = "🚨";
const SUCCESS_EMOJI: &str = "✅";
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

    failover_triggered_ms: u128,
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
    // this should never be freed by design
    let nodes: &[SentryClient] = config.nodes.iter().map(|node| SentryClient::new(node.clone(), Arc::clone(&config.http_client))).collect::<Vec<_>>().leak();
    let rpc_client = RpcClient::new_with_commitment(config.external_rpc.clone(), CommitmentConfig::processed());
    let vote_key = Pubkey::from_str(&config.vote_id).unwrap();
    let vote = VoteState::deserialize(&rpc_client.get_account(&vote_key).await.unwrap().data).unwrap();
    let identity = vote.authorized_voters().first().unwrap().1;
    let mut sanity_check_result = SanityCheckResult::default();
    let delinquency_ms_threshold = 60000u64.saturating_sub(400 * config.checks.unhealthy_threshold) as u128;

    let mut delinquent_triggered = false;
    let mut identity_balance_triggered = false;
    let mut rpc_unhealthy_triggered = false;

    print!("Delinquency threshold: {}ms\n", delinquency_ms_threshold);

    config.send_webhook(&format!("{} {}: Sentry is up and running!", INFO_EMOJI, identity.to_string())).await;
    loop {
        let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();

        let statuses: FuturesUnordered<_> = nodes.iter().map(|node| tokio::spawn(node.get_status())).collect();
        let (statuses, vote_account, identity_balance, ref_slot) = join!(
            futures::future::join_all(statuses),
            rpc_client.get_account(&vote_key),
            rpc_client.get_balance(identity),
            rpc_client.get_slot(),
        );
        let vote = VoteState::deserialize(&vote_account.unwrap().data).unwrap();
        let identity_balance = identity_balance.unwrap();
        let ref_slot = ref_slot.unwrap();

        // run sanity checks
        let identity_balance_low = identity_balance < config.checks.identity_balance;
        let node_slot = statuses.iter().map(|status| status.as_ref().unwrap().slot).max().unwrap();
        let rpc_unhealthy = ref_slot < node_slot - config.checks.unhealthy_threshold && node_slot > 0;
        // our best guess of the real time slot based on max(ref_slot, ..node_slots)
        let current_slot = max(ref_slot, node_slot);
        // distance of our last vote from the current slot, the minimum possible value is 1
        // we don't want RPC issues to trigger failover
        let vote_distance = ref_slot.saturating_sub(vote.votes[vote.votes.len() - 1].lockout.slot());
        let node_unhealthy = vote_distance > config.checks.unhealthy_threshold;

        let primary_node_status = match statuses.iter().filter(|x| x.as_ref().unwrap().identity == identity.to_string()).next() {
            Some(status) => status.as_ref().unwrap(),
            None => {
                &Status::unreachable()
            }
        };

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

            ..sanity_check_result
        };
        // println!("{}", sanity_check_result);
        let delinquent_for = now_ms - sanity_check_result.delinquent_since_ms;
        let delinquent_for_too_long = sanity_check_result.delinquent_since_ms > 0 && delinquent_for >= delinquency_ms_threshold;
        let nobody_voting = healthy_node_count == total_node_count && node_unhealthy && delinquent_for >= 5000;
        let identity_depleted = identity_balance < 895880;
        let no_failovers_recently = now_ms - sanity_check_result.failover_triggered_ms >= 10000;

        if delinquent_for_too_long {
            if !delinquent_triggered {
                delinquent_triggered = true;
                config.send_webhook(&format!("{} {}: Node is delinquent", ERROR_EMOJI, &identity.to_string())).await;
            }
        } else {
            if delinquent_triggered {
                delinquent_triggered = false;
                config.send_webhook(&format!("{} {}: Node is no longer delinquent", SUCCESS_EMOJI, &identity.to_string())).await;
            }
        }
        if identity_balance_low {
            if !identity_balance_triggered {
                identity_balance_triggered = true;
                config.send_webhook(&format!("{} {}: Identity balance is low - ◎{:.09}", WARNING_EMOJI, &identity.to_string(), (identity_balance as f64) / (LAMPORTS_PER_SOL as f64))).await;
            }
        } else {
            if identity_balance_triggered {
                identity_balance_triggered = false;
                config.send_webhook(&format!("{} {}: Identity balance is ok", SUCCESS_EMOJI, &identity.to_string())).await;
            }
        }
        if rpc_unhealthy {
            if !rpc_unhealthy_triggered {
                rpc_unhealthy_triggered = true;
                config.send_webhook(&format!("{} {}: RPC is unhealthy - {} slots behind",WARNING_EMOJI, &identity.to_string(), node_slot - ref_slot)).await;
            }
        } else {
            if rpc_unhealthy_triggered {
                rpc_unhealthy_triggered = false;
                config.send_webhook(&format!("{} {}: RPC is healthy", SUCCESS_EMOJI, &identity.to_string())).await;
            }
        }
        // delinquency can be due to multiple reasons
        // 1. identity_balance_low: not enough to pay for voting gas - nothing we can do here
        // 2. primary node is online but validator process is falling behind - covered by the nobody_voting check and we switch to a healthy node asap
        // 3. all nodes are on secondary identities - covered by the nobody_voting check
        // 4. primary node is powered on but internet access is lost - the monitoring will set the validator to secondary identity after 45secs of lost connectivity
        //    in the meantime the primary node will also be unreachable hence nobody_voting will be false, so after 60s of no voting we promote another node
        // 5. there're just no nodes online - cry, scream, panic
        // then after triggering a failover, we allow 10s for things to settle down
        if !identity_depleted && (delinquent_for_too_long || nobody_voting) && no_failovers_recently {
            // delinquent, trigger failover
            print!("Delinquent for {}ms, triggering failover\n", now_ms - sanity_check_result.delinquent_since_ms);
            if !primary_node_status.hostname.starts_with("(") {
                // if the primary node exists, set it to secondary
                print!("Setting primary node to secondary identity\n");
                SentryClient::new(primary_node_status.hostname.clone(), Arc::clone(&config.http_client)).set_identity(IdentityVariant::Secondary).await;
            }
            // find the node with the highest slot
            let new_primary = statuses.iter().max_by_key(|status| status.as_ref().unwrap().slot).unwrap().as_ref().unwrap();
            // since we're already delinquent, tower doesn't matter, just set_identity
            print!("Failing over to {}\n", new_primary.hostname);
            SentryClient::new(new_primary.hostname.clone(), Arc::clone(&config.http_client)).set_identity(IdentityVariant::Primary).await;
            sanity_check_result.failover_triggered_ms = now_ms;
            config.send_webhook(&format!("{} {}: Automatically failed over to {}", INFO_EMOJI, identity.to_string(), new_primary.hostname)).await;
        }
        let spent_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() - now_ms;
        if spent_ms < 2000 {
            thread::sleep(Duration::from_millis(2000u64.saturating_sub(spent_ms as u64)));
        }
    }
}