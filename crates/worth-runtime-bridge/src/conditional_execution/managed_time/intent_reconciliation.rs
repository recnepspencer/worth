use super::{
    contract::validate_identity, BridgeManagedTemporalDenial, BridgeManagedTemporalIntentLifecycle,
    BridgeManagedTemporalIntentReconciliation, BridgeManagedTemporalIntentReconciliationParts,
};
use crate::conditional_execution::BridgeOwnedSignalRuntime;

impl BridgeOwnedSignalRuntime {
    pub fn reconcile_managed_temporal_intent(
        &self,
        parts: BridgeManagedTemporalIntentReconciliationParts<'_>,
    ) -> Result<BridgeManagedTemporalIntentReconciliation, BridgeManagedTemporalDenial> {
        let cell = self.admit_managed_clock_lane(parts.binding)?;
        let mut lane = super::lock_lane(&cell)?;
        reconcile_intent_in_lane(&mut lane, parts)
    }
}

pub(in crate::conditional_execution) fn reconcile_intent_in_lane(
    lane: &mut super::BridgeManagedClockLane,
    parts: BridgeManagedTemporalIntentReconciliationParts<'_>,
) -> Result<BridgeManagedTemporalIntentReconciliation, BridgeManagedTemporalDenial> {
    validate_identity(parts.identity.as_str(), "temporal intent")?;
    validate_identity(&parts.idempotency_identity, "temporal intent idempotency")?;
    super::lifecycle::require_clock_lane(lane, parts.binding)?;
    match parts.lifecycle {
        BridgeManagedTemporalIntentLifecycle::Active => lane.reconcile_active_intent(
            parts.identity,
            parts.revision,
            parts.due_coordinate,
            parts.idempotency_identity,
            parts.source_record_identity,
        ),
        lifecycle @ (BridgeManagedTemporalIntentLifecycle::Cancelled
        | BridgeManagedTemporalIntentLifecycle::Completed) => {
            lane.reconcile_terminal_intent(&parts.identity, parts.revision, lifecycle)
        }
    }
}
