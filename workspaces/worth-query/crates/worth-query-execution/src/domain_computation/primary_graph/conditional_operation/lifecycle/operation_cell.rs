use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use std::sync::{Arc, Mutex, MutexGuard};

use worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly;

use super::{
    ConditionalClockLease, ErasedClockObservationOutcome, WorthQueryConditionalTruthBasis,
    WorthQueryInstalledConditionalOperation,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

/// Retained ownership of one installed operation. The lease is immutable and
/// lookup never needs to acquire the operation's execution mutex.
pub(in crate::domain_computation::primary_graph) struct WorthQueryConditionalOperationCell<Schema> {
    operation: Arc<Mutex<Box<dyn WorthQueryInstalledConditionalOperation<Schema>>>>,
    lease: Arc<ConditionalClockLease>,
}

impl<Schema> Clone for WorthQueryConditionalOperationCell<Schema> {
    fn clone(&self) -> Self {
        Self {
            operation: Arc::clone(&self.operation),
            lease: Arc::clone(&self.lease),
        }
    }
}

impl<Schema> WorthQueryConditionalOperationCell<Schema> {
    pub(super) fn new(operation: Box<dyn WorthQueryInstalledConditionalOperation<Schema>>) -> Self {
        Self {
            lease: operation.clock_lease(),
            operation: Arc::new(Mutex::new(operation)),
        }
    }

    pub(super) fn matches_clock_lease(&self, lease: &Arc<ConditionalClockLease>) -> bool {
        Arc::ptr_eq(&self.lease, lease)
    }

    pub(in crate::domain_computation::primary_graph) fn operation_anchor(
        &self,
    ) -> Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering> {
        self.lock_operation().operation_anchor()
    }

    pub(in crate::domain_computation::primary_graph) fn lock_operation(
        &self,
    ) -> MutexGuard<'_, Box<dyn WorthQueryInstalledConditionalOperation<Schema>>> {
        self.operation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(in crate::domain_computation::primary_graph::conditional_operation) fn observe_clock(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

        bridge: &BridgeSealedRuntimeAssembly,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        truth: &WorthQueryConditionalTruthBasis,
    ) -> ErasedClockObservationOutcome {
        let mut operation = self.lock_operation();
        if let Err(denial) = operation.select_product_binding(bridge, runtime, truth) {
            return ErasedClockObservationOutcome::Failed {
                kind: installation_denial_observation_kind(denial.kind()),
                detail: denial.subject().to_string(),
            };
        }
        operation.observe_clock(phase, bridge, runtime, truth)
    }

    pub(in crate::domain_computation::primary_graph::conditional_operation) fn admit_product_binding(
        &self,
        bridge: &BridgeSealedRuntimeAssembly,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        truth: &WorthQueryConditionalTruthBasis,
    ) -> Result<(), super::super::installation::WorthQueryConditionalRuntimeInstallationDenial>
    {
        self.lock_operation()
            .select_product_binding(bridge, runtime, truth)
    }
}

fn installation_denial_observation_kind(
    kind: super::super::installation::WorthQueryConditionalRuntimeInstallationDenialKind,
) -> super::WorthQueryConditionalClockObservationFailureKind {
    match kind {
        super::super::installation::WorthQueryConditionalRuntimeInstallationDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => super::WorthQueryConditionalClockObservationFailureKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        _ => super::WorthQueryConditionalClockObservationFailureKind::RuntimeRejected,
    }
}
