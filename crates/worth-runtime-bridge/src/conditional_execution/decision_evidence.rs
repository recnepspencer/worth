use worth_signal::facade::adapters::SignalInvalidationExecutionReceipt;
use worth_signal::facade::SignalConditionalDecisionEvidence;

use super::execution::{
    BridgeConditionalQueryContinuationAdmission, BridgeConditionalReentryCounters,
};
use super::retained_decision::BridgeRetainedConditionalDecisionCore;
use super::BridgeInstalledConditionalLowering;

pub struct BridgeConditionalDecisionEvidence {
    pub(super) core: std::sync::Arc<BridgeRetainedConditionalDecisionCore>,
    pub(super) query_binding_identity: std::sync::Arc<str>,
    pub(super) query_capability_identity: u64,
    pub(super) reentry_counters: BridgeConditionalReentryCounters,
    pub(super) performed_signal_invalidation: Option<SignalInvalidationExecutionReceipt>,
    pub(super) _reservation: super::retention::BridgeRetentionReservation,
}

impl BridgeConditionalDecisionEvidence {
    pub fn lowering_projection(&self) -> &super::BridgeConditionalLoweringProjectionIdentity {
        self.core.lowering.projection()
    }

    pub fn retains_exact_lowering(
        &self,
        lowering: &std::sync::Arc<BridgeInstalledConditionalLowering>,
    ) -> bool {
        std::sync::Arc::ptr_eq(&self.core.lowering, lowering)
    }

    pub fn query_binding_identity(&self) -> &str {
        &self.query_binding_identity
    }

    pub const fn query_capability_identity(&self) -> u64 {
        self.query_capability_identity
    }

    pub fn signal_snapshot_projection(&self) -> &str {
        &self.core.signal_snapshot_projection
    }

    pub fn signal_execution_projection(&self) -> &str {
        &self.core.signal_execution_projection
    }

    pub fn attempt(&self) -> u64 {
        self.core.attempt
    }

    pub fn bridge_snapshot_identity(&self) -> Option<&crate::snapshot::TruthSnapshotIdentity> {
        self.core.bridge_snapshot_identity()
    }

    pub fn retains_bridge_snapshot_identity(
        &self,
        candidate: &crate::snapshot::TruthSnapshotIdentity,
    ) -> bool {
        self.core.bridge_snapshot_identity() == Some(candidate)
    }

    pub fn signal(&self) -> &SignalConditionalDecisionEvidence {
        &self.core.signal
    }

    pub fn semantic_observation_reads(&self) -> usize {
        self.core.semantic_observations.len()
    }

    pub fn semantic_observations(&self) -> &[super::BridgeConditionalSemanticObservation] {
        &self.core.semantic_observations
    }

    pub fn bridge_execution_counters(&self) -> super::BridgeConditionalExecutionCounters {
        self.core.bridge_execution_counters
    }

    pub const fn reentry_counters(&self) -> BridgeConditionalReentryCounters {
        self.reentry_counters
    }

    pub fn performed_signal_invalidation(&self) -> Option<&SignalInvalidationExecutionReceipt> {
        self.performed_signal_invalidation.as_ref()
    }

    pub(crate) fn signal_graph_instance_id(&self) -> u64 {
        self.core.lowering.signal_graph_instance_id()
    }

    pub(crate) fn signal_node(&self) -> worth_signal::facade::NodeId {
        self.core.lowering.signal_node()
    }

    pub(crate) fn take_performed_signal_invalidation(
        &mut self,
    ) -> Option<SignalInvalidationExecutionReceipt> {
        self.performed_signal_invalidation.take()
    }

    pub(crate) fn retains_triggering_correspondence(
        &self,
        candidate: &crate::correspondence::BridgeDeliveredCorrespondenceChangeSet,
    ) -> bool {
        self.core
            .triggering_change_set
            .as_ref()
            .is_some_and(|retained| retained.retains_same_delivery_as(candidate))
    }

    pub fn admits_query_continuation(
        &self,
        admission: BridgeConditionalQueryContinuationAdmission<'_>,
    ) -> bool {
        self.retains_exact_lowering(admission.lowering)
            && self.query_binding_identity.as_ref() == admission.query_binding_identity
            && self.query_capability_identity == admission.query_capability_identity
            && self.core.signal_snapshot_projection.as_ref() == admission.signal_snapshot_projection
            && self.core.bridge_snapshot_identity() == admission.bridge_snapshot_identity
            && self.core.signal_execution_projection.as_ref()
                == admission.signal_execution_projection
            && self.core.attempt == admission.attempt
            && admission
                .lowering
                .validate_signal_decision_contract(&self.core.signal)
                .is_ok()
    }
}
