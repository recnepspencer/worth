use std::sync::Arc;

use crate::input::envelope::BridgeCommittedPatchEnvelope;
use crate::subscription::BridgeSubscriptionDeliveryFamilyKind;

use super::{
    cause::BridgeTemporalCauseRecordInput, BridgeTemporalCauseClassification,
    BridgeTemporalCauseRecord, BridgeTemporalDeliveryWindowPlan,
    BridgeTemporalWakeRoutingRejection, BridgeTemporalWakeRoutingRejectionKind,
    BridgeTemporalWakeRoutingRequest,
};

impl BridgeTemporalCauseRecord {
    pub(crate) fn route_wake(
        request: &BridgeTemporalWakeRoutingRequest,
        prior_cause: Option<&Self>,
    ) -> Result<Self, BridgeTemporalWakeRoutingRejection> {
        route_temporal_cause(request, None, prior_cause)
    }

    pub(crate) fn route_wake_with_truth_patch(
        request: &BridgeTemporalWakeRoutingRequest,
        truth_patch: &BridgeCommittedPatchEnvelope,
        prior_cause: Option<&Self>,
    ) -> Result<Self, BridgeTemporalWakeRoutingRejection> {
        route_temporal_cause(request, Some(truth_patch), prior_cause)
    }

    pub(crate) fn plan_delivery_window(
        &self,
        delivery_family_kind: BridgeSubscriptionDeliveryFamilyKind,
    ) -> BridgeTemporalDeliveryWindowPlan {
        BridgeTemporalDeliveryWindowPlan::plan(self, delivery_family_kind)
    }
}

fn route_temporal_cause(
    request: &BridgeTemporalWakeRoutingRequest,
    truth_patch: Option<&BridgeCommittedPatchEnvelope>,
    prior_cause: Option<&BridgeTemporalCauseRecord>,
) -> Result<BridgeTemporalCauseRecord, BridgeTemporalWakeRoutingRejection> {
    if let Some(prior_cause) = prior_cause {
        if prior_cause.routing_lane_kind() != request.routing_lane_kind()
            || prior_cause.subscription_identity() != request.subscription_identity()
            || prior_cause.preview_basis_identity() != request.preview_basis_identity()
        {
            return Err(BridgeTemporalWakeRoutingRejection::new(
                BridgeTemporalWakeRoutingRejectionKind::RoutingLaneMismatch,
                request.routing_lane_kind(),
                request.activation_lane_identity(),
                request.temporal_basis_identity().as_str(),
                request.wake_id(),
                request.wake_tick(),
                truth_patch
                    .map(|patch| patch.patch_identity().canonical_label())
                    .as_deref(),
            ));
        }

        if prior_cause.wake_tick() > request.wake_tick() {
            return Err(BridgeTemporalWakeRoutingRejection::new(
                BridgeTemporalWakeRoutingRejectionKind::StaleWakeSubmission,
                request.routing_lane_kind(),
                request.activation_lane_identity(),
                request.temporal_basis_identity().as_str(),
                request.wake_id(),
                request.wake_tick(),
                truth_patch
                    .map(|patch| patch.patch_identity().canonical_label())
                    .as_deref(),
            ));
        }

        if prior_cause.wake_tick() == request.wake_tick()
            || prior_cause.wake_id() == request.wake_id()
        {
            return Err(BridgeTemporalWakeRoutingRejection::new(
                BridgeTemporalWakeRoutingRejectionKind::DuplicateWakeSubmission,
                request.routing_lane_kind(),
                request.activation_lane_identity(),
                request.temporal_basis_identity().as_str(),
                request.wake_id(),
                request.wake_tick(),
                truth_patch
                    .map(|patch| patch.patch_identity().canonical_label())
                    .as_deref(),
            ));
        }
    }

    if let Some(truth_patch) = truth_patch {
        if truth_patch.snapshot_identity() != request.truth_snapshot_identity() {
            return Err(BridgeTemporalWakeRoutingRejection::new(
                BridgeTemporalWakeRoutingRejectionKind::TruthPatchSnapshotIdentityMismatch,
                request.routing_lane_kind(),
                request.activation_lane_identity(),
                request.temporal_basis_identity().as_str(),
                request.wake_id(),
                request.wake_tick(),
                Some(&truth_patch.patch_identity().canonical_label()),
            ));
        }
        if truth_patch.branch_identity() != request.truth_branch_identity() {
            return Err(BridgeTemporalWakeRoutingRejection::new(
                BridgeTemporalWakeRoutingRejectionKind::TruthPatchBranchIdentityMismatch,
                request.routing_lane_kind(),
                request.activation_lane_identity(),
                request.temporal_basis_identity().as_str(),
                request.wake_id(),
                request.wake_tick(),
                Some(&truth_patch.patch_identity().canonical_label()),
            ));
        }
    }

    Ok(BridgeTemporalCauseRecord::route(
        BridgeTemporalCauseRecordInput {
            routing_lane_kind: request.routing_lane_kind(),
            subscription_identity: Arc::from(request.subscription_identity().to_owned()),
            activation_lane_identity: Arc::from(request.activation_lane_identity().to_owned()),
            temporal_basis_identity: request.temporal_basis_identity().clone(),
            preview_basis_identity: request.preview_basis_identity().cloned(),
            classification: if truth_patch.is_some() {
                BridgeTemporalCauseClassification::TruthPlusTime
            } else {
                BridgeTemporalCauseClassification::TimeOnly
            },
            wake_id: request.wake_id(),
            wake_ready_ordinal: request.wake_ready_ordinal(),
            wake_tick: request.wake_tick(),
            truth_patch_identity: truth_patch.map(|patch| patch.patch_identity().clone()),
            truth_patch_digest: truth_patch
                .map(|patch| Arc::from(patch.digest().as_str().to_owned())),
        },
    ))
}
