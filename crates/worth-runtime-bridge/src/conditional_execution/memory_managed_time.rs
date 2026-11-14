//! Memory receivers borrow their original common execution view for managed time.
use super::*;
#[cfg(test)]
use std::sync::{Arc, Mutex};
impl BridgeOwnedSignalRuntime {
    pub fn install_managed_clock(
        &self,
        parts: BridgeManagedClockInstallationParts<'_>,
    ) -> Result<BridgeManagedClockBinding, BridgeManagedTemporalDenial> {
        self.execution_view().install_managed_clock(parts)
    }
    pub fn close_managed_clock(
        &self,
        binding: BridgeManagedClockBinding,
    ) -> Result<BridgeManagedClockClosure, BridgeManagedTemporalDenial> {
        self.execution_view().close_managed_clock(&binding)
    }
    pub fn observe_managed_clock(
        &self,
        parts: BridgeManagedClockObservationParts<'_>,
    ) -> Result<BridgeManagedClockObservationOutcome, BridgeManagedTemporalDenial> {
        self.execution_view().observe_managed_clock(parts)
    }
    pub fn reconcile_managed_temporal_intent(
        &self,
        parts: BridgeManagedTemporalIntentReconciliationParts<'_>,
    ) -> Result<BridgeManagedTemporalIntentReconciliation, BridgeManagedTemporalDenial> {
        self.execution_view()
            .reconcile_managed_temporal_intent(parts)
    }
    pub fn execute_managed_due_wake(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        request: BridgeManagedConditionalExecutionRequest<'_>,
        context: &mut dyn std::any::Any,
    ) -> Result<BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.execution_view()
            .execute_managed_due_wake(execution, request, context)
    }
    #[cfg(test)]
    pub(in crate::conditional_execution) fn admit_managed_clock_lane(
        &self,
        binding: &BridgeManagedClockBinding,
    ) -> Result<Arc<Mutex<managed_time::BridgeManagedClockLane>>, BridgeManagedTemporalDenial> {
        self.execution_view().admit_managed_clock_lane(binding)
    }
    pub(in crate::conditional_execution) fn revoke_managed_clock_liveness(&self) {
        self.execution_view().revoke_managed_clock_liveness()
    }
    pub fn managed_clock_count(&self) -> usize {
        self.execution_view().managed_clock_count()
    }
}
