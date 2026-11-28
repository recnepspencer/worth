use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_signal::facade::{ClockTick, TemporalWakeId, WakeOrdinal};

use crate::input::envelope::TruthBranchIdentity;
use crate::snapshot::TruthSnapshotIdentity;
use crate::subscription::{
    BridgePreviewTemporalSubscriptionActivationReady, BridgeSubscriptionPreviewBasisIdentity,
    BridgeSubscriptionTemporalWakeRoutingRequestIdentity,
    BridgeTemporalSubscriptionActivationReady,
};
use crate::temporal::BridgeTemporalBasisIdentity;

use super::BridgeTemporalRoutingLaneKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeTemporalWakeRoutingRequest {
    routing_request_identity: BridgeSubscriptionTemporalWakeRoutingRequestIdentity,
    routing_lane_kind: BridgeTemporalRoutingLaneKind,
    subscription_identity: Arc<str>,
    activation_lane_identity: Arc<str>,
    temporal_basis_identity: BridgeTemporalBasisIdentity,
    preview_basis_identity: Option<BridgeSubscriptionPreviewBasisIdentity>,
    truth_branch_identity: TruthBranchIdentity,
    truth_snapshot_identity: TruthSnapshotIdentity,
    wake_id: TemporalWakeId,
    wake_ready_ordinal: WakeOrdinal,
    wake_tick: ClockTick,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeTemporalWakeRoutingRequest {
    pub(crate) fn prepare_authoritative(
        activation_ready: &BridgeTemporalSubscriptionActivationReady,
    ) -> Self {
        let temporal_basis = activation_ready.temporal_admission().temporal_basis();
        let truth_basis = temporal_basis.truth_basis().basis();
        let wake = temporal_basis.wake_evidence().evidence();
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-temporal-wake-routing-request|lane=authoritative|subscription={}|activation={}|temporal-basis={}|truth-branch={}|truth-snapshot={}|wake-id={}|wake-ready-ordinal={}|wake-tick={}",
            activation_ready
                .ordinary_activation_ready()
                .admitted()
                .admitted_subscription_identity()
                .as_str(),
            activation_ready.temporal_activation_ready_identity().as_str(),
            temporal_basis.identity().as_str(),
            truth_basis.branch_identity().canonical_label(),
            truth_basis.snapshot_identity().canonical_label(),
            wake.wake_id().get(),
            wake.wake_ready_ordinal().get(),
            wake.wake_tick().get(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            routing_request_identity:
                BridgeSubscriptionTemporalWakeRoutingRequestIdentity::admit_bridge_owned(format!(
                    "bridge-temporal-wake-routing-request-id:sha256:{digest:x}"
                )),
            routing_lane_kind: BridgeTemporalRoutingLaneKind::Authoritative,
            subscription_identity: Arc::from(
                activation_ready
                    .ordinary_activation_ready()
                    .admitted()
                    .admitted_subscription_identity()
                    .as_str()
                    .to_owned(),
            ),
            activation_lane_identity: Arc::from(
                activation_ready
                    .temporal_activation_ready_identity()
                    .as_str()
                    .to_owned(),
            ),
            temporal_basis_identity: temporal_basis.identity().clone(),
            preview_basis_identity: None,
            truth_branch_identity: truth_basis.branch_identity().clone(),
            truth_snapshot_identity: truth_basis.snapshot_identity().clone(),
            wake_id: wake.wake_id(),
            wake_ready_ordinal: wake.wake_ready_ordinal(),
            wake_tick: wake.wake_tick(),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-temporal-wake-routing-request:sha256:{digest:x}"
            )),
        }
    }

    pub(crate) fn prepare_preview(
        activation_ready: &BridgePreviewTemporalSubscriptionActivationReady,
    ) -> Self {
        let temporal_basis = activation_ready
            .preview_temporal_admission()
            .temporal_basis();
        let truth_basis = temporal_basis.truth_basis().basis();
        let wake = temporal_basis.wake_evidence().evidence();
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-temporal-wake-routing-request|lane=preview|subscription={}|activation={}|temporal-basis={}|preview-basis={}|truth-branch={}|truth-snapshot={}|wake-id={}|wake-ready-ordinal={}|wake-tick={}",
            activation_ready
                .ordinary_activation_ready()
                .admitted()
                .admitted_subscription_identity()
                .as_str(),
            activation_ready
                .preview_temporal_activation_ready_identity()
                .as_str(),
            temporal_basis.identity().as_str(),
            activation_ready
                .preview_temporal_admission()
                .preview_basis()
                .preview_basis_identity()
                .as_str(),
            truth_basis.branch_identity().canonical_label(),
            truth_basis.snapshot_identity().canonical_label(),
            wake.wake_id().get(),
            wake.wake_ready_ordinal().get(),
            wake.wake_tick().get(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            routing_request_identity:
                BridgeSubscriptionTemporalWakeRoutingRequestIdentity::admit_bridge_owned(format!(
                    "bridge-temporal-wake-routing-request-id:sha256:{digest:x}"
                )),
            routing_lane_kind: BridgeTemporalRoutingLaneKind::Preview,
            subscription_identity: Arc::from(
                activation_ready
                    .ordinary_activation_ready()
                    .admitted()
                    .admitted_subscription_identity()
                    .as_str()
                    .to_owned(),
            ),
            activation_lane_identity: Arc::from(
                activation_ready
                    .preview_temporal_activation_ready_identity()
                    .as_str()
                    .to_owned(),
            ),
            temporal_basis_identity: temporal_basis.identity().clone(),
            preview_basis_identity: Some(
                activation_ready
                    .preview_temporal_admission()
                    .preview_basis()
                    .preview_basis_identity()
                    .clone(),
            ),
            truth_branch_identity: truth_basis.branch_identity().clone(),
            truth_snapshot_identity: truth_basis.snapshot_identity().clone(),
            wake_id: wake.wake_id(),
            wake_ready_ordinal: wake.wake_ready_ordinal(),
            wake_tick: wake.wake_tick(),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-temporal-wake-routing-request:sha256:{digest:x}"
            )),
        }
    }

    pub fn routing_lane_kind(&self) -> BridgeTemporalRoutingLaneKind {
        self.routing_lane_kind
    }

    pub(crate) fn subscription_identity(&self) -> &str {
        self.subscription_identity.as_ref()
    }

    pub(crate) fn activation_lane_identity(&self) -> &str {
        self.activation_lane_identity.as_ref()
    }

    pub(crate) fn temporal_basis_identity(&self) -> &BridgeTemporalBasisIdentity {
        &self.temporal_basis_identity
    }

    pub(crate) fn preview_basis_identity(&self) -> Option<&BridgeSubscriptionPreviewBasisIdentity> {
        self.preview_basis_identity.as_ref()
    }

    pub(crate) fn truth_branch_identity(&self) -> &TruthBranchIdentity {
        &self.truth_branch_identity
    }

    pub(crate) fn truth_snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.truth_snapshot_identity
    }

    pub(crate) const fn wake_id(&self) -> TemporalWakeId {
        self.wake_id
    }

    pub(crate) const fn wake_ready_ordinal(&self) -> WakeOrdinal {
        self.wake_ready_ordinal
    }

    pub(crate) const fn wake_tick(&self) -> ClockTick {
        self.wake_tick
    }
}
