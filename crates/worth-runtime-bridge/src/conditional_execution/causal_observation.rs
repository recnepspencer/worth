//! Read-only custody of an actual conditional observation, without its reader.
use std::sync::Arc;

use super::{
    BridgeConditionalLoweringProjectionIdentity, BridgeConditionalSemanticObservation,
    BridgeOwnedSignalRuntime,
};

mod retained;
pub(super) use retained::BridgeRetainedConditionalObservation;

/// Evidence from one actual Native evaluation. This object grants no execution,
/// reentry, source-read or disclosure permission, and retains no snapshot reader.
/// Its immutable backing keeps the original Native retention reservation.
#[derive(Clone)]
pub struct BridgeConditionalCausalObservation {
    observation: Arc<BridgeRetainedConditionalObservation>,
}

impl BridgeConditionalCausalObservation {
    pub(super) fn from_issued(observation: Arc<BridgeRetainedConditionalObservation>) -> Self {
        Self { observation }
    }

    /// Reads only the actual optional history owner; immutable Native custody is separate.
    pub fn history_availability(&self) -> crate::diagnostics::BridgeConditionalHistoryAvailability {
        self.observation.history.availability()
    }
    /// Rechecks the history owner now; earlier availability is not a retention lease.
    pub fn history_detail(
        &self,
    ) -> Result<
        crate::diagnostics::BridgeConditionalHistoryDetail,
        crate::diagnostics::BridgeConditionalHistoryAvailability,
    > {
        self.observation.history.detail()
    }

    pub fn belongs_to(&self, owner: &BridgeOwnedSignalRuntime) -> bool {
        self.observation.bridge_runtime_key == owner.bridge.signal_runtime_key
    }

    pub fn same_observation(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.observation, &other.observation)
    }

    pub fn lowering_projection(&self) -> &BridgeConditionalLoweringProjectionIdentity {
        &self.observation.lowering_projection
    }

    pub fn signal_projection(
        &self,
    ) -> &worth_signal::facade::SignalConditionalDecisionProjectionIdentity {
        self.observation.signal.projection()
    }

    pub fn signal_class(&self) -> worth_signal::facade::SignalConditionalDecisionClass {
        self.observation.signal.class()
    }

    /// Borrows the original Signal computed effect; passive decisions have none.
    /// Its meaning belongs to this exact installed lowering and source observation.
    pub fn computed_evaluation(
        &self,
    ) -> Option<&worth_signal::facade::diagnostics::AppliedEffectReport> {
        self.observation.signal.computed_evaluation()
    }

    pub fn signal_output_versions(&self) -> (u64, u64) {
        (
            self.observation.signal.output_version_before(),
            self.observation.signal.output_version(),
        )
    }

    pub fn attempt(&self) -> u64 {
        self.observation.attempt
    }

    pub fn bridge_snapshot_identity(&self) -> Option<&crate::snapshot::TruthSnapshotIdentity> {
        self.observation.bridge_snapshot_identity.as_ref()
    }

    pub fn semantic_observations(&self) -> &[BridgeConditionalSemanticObservation] {
        &self.observation.semantic_observations
    }

    pub fn performed_signal_invalidation(
        &self,
    ) -> Option<&worth_signal::facade::adapters::SignalInvalidationCausalObservation> {
        self.observation.performed_signal_invalidation.as_ref()
    }
}

impl std::fmt::Debug for BridgeConditionalCausalObservation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BridgeConditionalCausalObservation")
            .finish_non_exhaustive()
    }
}
