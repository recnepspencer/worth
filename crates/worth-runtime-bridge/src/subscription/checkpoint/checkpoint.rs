use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::super::{
    BridgeActiveSubscription, BridgeActiveSubscriptionIdentity, BridgeAdmittedSubscriptionIdentity,
    BridgeSubscriptionBasisIdentity, BridgeSubscriptionCheckpointIdentity,
    BridgeSubscriptionCheckpointReadyIdentity, BridgeSubscriptionConsumerContractIdentity,
    BridgeSubscriptionCounters, BridgeSubscriptionDeliveryCostProfileIdentity,
    BridgeSubscriptionDeliveryDiagnosticsReferenceIdentity,
    BridgeSubscriptionDeliveryFamilyIdentity, BridgeSubscriptionDeliveryMemberIdentity,
    BridgeSubscriptionDeliveryWindowIdentity, BridgeSubscriptionFanoutLayout,
    BridgeSubscriptionFanoutLayoutIdentity,
};
use super::{
    BridgeSubscriptionCheckpointReady, BridgeSubscriptionCheckpointRejection,
    BridgeSubscriptionCheckpointRejectionKind, BridgeSubscriptionDuplicateReplayPolicy,
    BridgeSubscriptionDuplicateReplayPolicyKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionCheckpoint {
    checkpoint_identity: BridgeSubscriptionCheckpointIdentity,
    checkpoint_ready_identity: BridgeSubscriptionCheckpointReadyIdentity,
    active_subscription_identity: BridgeActiveSubscriptionIdentity,
    admitted_subscription_identity: BridgeAdmittedSubscriptionIdentity,
    delivery_family_identity: BridgeSubscriptionDeliveryFamilyIdentity,
    delivery_window_identity: BridgeSubscriptionDeliveryWindowIdentity,
    delivery_window_sequence: u64,
    basis_identity: BridgeSubscriptionBasisIdentity,
    acknowledged_canonical_sequence: usize,
    acknowledged_member_identity: BridgeSubscriptionDeliveryMemberIdentity,
    acknowledged_member_digest: Arc<str>,
    acknowledged_prefix_digest: Arc<str>,
    cost_profile_identity: BridgeSubscriptionDeliveryCostProfileIdentity,
    consumer_contract_identity: BridgeSubscriptionConsumerContractIdentity,
    fanout_layout_identity: Option<BridgeSubscriptionFanoutLayoutIdentity>,
    duplicate_replay_policy: BridgeSubscriptionDuplicateReplayPolicy,
    diagnostics_reference_identity: BridgeSubscriptionDeliveryDiagnosticsReferenceIdentity,
    counter_digest: Arc<str>,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeSubscriptionCheckpoint {
    pub(crate) fn publish(
        ready: BridgeSubscriptionCheckpointReady,
        active_subscription: &BridgeActiveSubscription,
        duplicate_replay_policy_kind: BridgeSubscriptionDuplicateReplayPolicyKind,
        fanout_layout: Option<&BridgeSubscriptionFanoutLayout>,
    ) -> Result<
        Self,
        (
            BridgeSubscriptionCheckpointReady,
            BridgeSubscriptionCheckpointRejection,
        ),
    > {
        let frontier = ready.frontier();
        if frontier.active_subscription_identity()
            != active_subscription.active_subscription_identity()
        {
            let denial = BridgeSubscriptionCheckpointRejection::new(
                BridgeSubscriptionCheckpointRejectionKind::ActiveSubscriptionMismatch,
            );
            return Err((ready, denial));
        }
        let admitted = active_subscription.activation_ready().admitted();
        if frontier.admitted_subscription_identity() != admitted.admitted_subscription_identity() {
            let denial = BridgeSubscriptionCheckpointRejection::new(
                BridgeSubscriptionCheckpointRejectionKind::AdmittedSubscriptionMismatch,
            );
            return Err((ready, denial));
        }
        if frontier.basis_identity() != admitted.basis_binding().basis_identity() {
            let denial = BridgeSubscriptionCheckpointRejection::new(
                BridgeSubscriptionCheckpointRejectionKind::BasisMismatch,
            );
            return Err((ready, denial));
        }
        if let Some(layout) = fanout_layout {
            if layout.active_subscription_identity()
                != active_subscription.active_subscription_identity()
            {
                let denial = BridgeSubscriptionCheckpointRejection::new(
                    BridgeSubscriptionCheckpointRejectionKind::FanoutLayoutActiveSubscriptionMismatch,
                );
                return Err((ready, denial));
            }
            if layout.cost_profile_identity()
                != active_subscription.cost_profile().cost_profile_identity()
            {
                let denial = BridgeSubscriptionCheckpointRejection::new(
                    BridgeSubscriptionCheckpointRejectionKind::FanoutLayoutCostProfileMismatch,
                );
                return Err((ready, denial));
            }
            if layout.delivery_family().delivery_family_identity()
                != frontier.delivery_family_identity()
            {
                let denial = BridgeSubscriptionCheckpointRejection::new(
                    BridgeSubscriptionCheckpointRejectionKind::FanoutLayoutDeliveryFamilyMismatch,
                );
                return Err((ready, denial));
            }
        }
        let duplicate_replay_policy =
            BridgeSubscriptionDuplicateReplayPolicy::select(duplicate_replay_policy_kind);
        let fanout_layout_identity =
            fanout_layout.map(|layout| layout.fanout_layout_identity().clone());
        let fanout_basis = fanout_layout_identity
            .as_ref()
            .map(BridgeSubscriptionFanoutLayoutIdentity::as_str)
            .unwrap_or("none");
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-subscription-checkpoint|ready={}|active={}|admitted={}|family={}|window={}|sequence={}|basis={}|ack-sequence={}|ack-member={}|ack-digest={}|ack-prefix={}|cost={}|consumer={}|fanout={}|duplicate-policy={}|diagnostics={}|counter={}",
            ready.checkpoint_ready_identity().as_str(),
            frontier.active_subscription_identity().as_str(),
            frontier.admitted_subscription_identity().as_str(),
            frontier.delivery_family_identity().as_str(),
            frontier.delivery_window_identity().as_str(),
            frontier.delivery_window_sequence(),
            frontier.basis_identity().as_str(),
            frontier.acknowledged_canonical_sequence(),
            frontier.acknowledged_member_identity().as_str(),
            frontier.acknowledged_member_digest(),
            frontier.acknowledged_prefix_digest(),
            active_subscription.cost_profile().cost_profile_identity().as_str(),
            active_subscription.consumer_contract().consumer_contract_identity().as_str(),
            fanout_basis,
            duplicate_replay_policy.duplicate_replay_policy_identity().as_str(),
            frontier.diagnostics_reference_identity().as_str(),
            frontier.counter_digest(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Ok(Self {
            checkpoint_identity: BridgeSubscriptionCheckpointIdentity::admit_bridge_owned(format!(
                "bridge-subscription-checkpoint-id:sha256:{digest:x}"
            )),
            checkpoint_ready_identity: ready.checkpoint_ready_identity().clone(),
            active_subscription_identity: frontier.active_subscription_identity().clone(),
            admitted_subscription_identity: frontier.admitted_subscription_identity().clone(),
            delivery_family_identity: frontier.delivery_family_identity().clone(),
            delivery_window_identity: frontier.delivery_window_identity().clone(),
            delivery_window_sequence: frontier.delivery_window_sequence(),
            basis_identity: frontier.basis_identity().clone(),
            acknowledged_canonical_sequence: frontier.acknowledged_canonical_sequence(),
            acknowledged_member_identity: frontier.acknowledged_member_identity().clone(),
            acknowledged_member_digest: Arc::from(frontier.acknowledged_member_digest().to_owned()),
            acknowledged_prefix_digest: Arc::from(frontier.acknowledged_prefix_digest().to_owned()),
            cost_profile_identity: active_subscription
                .cost_profile()
                .cost_profile_identity()
                .clone(),
            consumer_contract_identity: active_subscription
                .consumer_contract()
                .consumer_contract_identity()
                .clone(),
            fanout_layout_identity,
            duplicate_replay_policy,
            diagnostics_reference_identity: frontier.diagnostics_reference_identity().clone(),
            counter_digest: Arc::from(frontier.counter_digest().to_owned()),
            counters: BridgeSubscriptionCounters::from_checkpoint_publication(),
            canonical_basis,
            digest: Arc::from(format!("bridge-subscription-checkpoint:sha256:{digest:x}")),
        })
    }

    pub fn checkpoint_identity(&self) -> &BridgeSubscriptionCheckpointIdentity {
        &self.checkpoint_identity
    }

    pub fn checkpoint_ready_identity(&self) -> &BridgeSubscriptionCheckpointReadyIdentity {
        &self.checkpoint_ready_identity
    }

    pub fn active_subscription_identity(&self) -> &BridgeActiveSubscriptionIdentity {
        &self.active_subscription_identity
    }

    pub fn admitted_subscription_identity(&self) -> &BridgeAdmittedSubscriptionIdentity {
        &self.admitted_subscription_identity
    }

    pub fn delivery_family_identity(&self) -> &BridgeSubscriptionDeliveryFamilyIdentity {
        &self.delivery_family_identity
    }

    pub fn delivery_window_identity(&self) -> &BridgeSubscriptionDeliveryWindowIdentity {
        &self.delivery_window_identity
    }

    pub fn delivery_window_sequence(&self) -> u64 {
        self.delivery_window_sequence
    }

    pub fn basis_identity(&self) -> &BridgeSubscriptionBasisIdentity {
        &self.basis_identity
    }

    pub fn cost_profile_identity(&self) -> &BridgeSubscriptionDeliveryCostProfileIdentity {
        &self.cost_profile_identity
    }

    pub fn consumer_contract_identity(&self) -> &BridgeSubscriptionConsumerContractIdentity {
        &self.consumer_contract_identity
    }

    pub fn acknowledged_canonical_sequence(&self) -> usize {
        self.acknowledged_canonical_sequence
    }

    pub fn acknowledged_member_identity(&self) -> &BridgeSubscriptionDeliveryMemberIdentity {
        &self.acknowledged_member_identity
    }

    pub fn acknowledged_member_digest(&self) -> &str {
        self.acknowledged_member_digest.as_ref()
    }

    pub fn acknowledged_prefix_digest(&self) -> &str {
        self.acknowledged_prefix_digest.as_ref()
    }

    pub fn duplicate_replay_policy(&self) -> &BridgeSubscriptionDuplicateReplayPolicy {
        &self.duplicate_replay_policy
    }

    pub fn fanout_layout_identity(&self) -> Option<&BridgeSubscriptionFanoutLayoutIdentity> {
        self.fanout_layout_identity.as_ref()
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
