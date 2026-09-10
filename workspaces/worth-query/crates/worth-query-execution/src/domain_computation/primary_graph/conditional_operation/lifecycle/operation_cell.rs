use std::sync::{Arc, Mutex, MutexGuard};

use worth_relational::facade::transactions::RecordRef;
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
    routes: Arc<Mutex<ConditionalOperationRoutes>>,
}

#[derive(Default)]
struct ConditionalOperationRoutes {
    records: Vec<RecordRef>,
    whole_graph: bool,
    bootstrap: bool,
}

pub(in crate::domain_computation::primary_graph::conditional_operation) struct WorthQueryObservedConditionalOperation
{
    pub(in crate::domain_computation::primary_graph::conditional_operation) outcome:
        ErasedClockObservationOutcome,
    pub(in crate::domain_computation::primary_graph::conditional_operation) routes_changed: bool,
}

impl<Schema> Clone for WorthQueryConditionalOperationCell<Schema> {
    fn clone(&self) -> Self {
        Self {
            operation: Arc::clone(&self.operation),
            lease: Arc::clone(&self.lease),
            routes: Arc::clone(&self.routes),
        }
    }
}

impl<Schema> WorthQueryConditionalOperationCell<Schema> {
    pub(super) fn new(operation: Box<dyn WorthQueryInstalledConditionalOperation<Schema>>) -> Self {
        let (records, whole_graph) = operation.authoritative_commit_routes();
        let bootstrap = operation.bootstrap_commit_route_pending();
        Self {
            lease: operation.clock_lease(),
            operation: Arc::new(Mutex::new(operation)),
            routes: Arc::new(Mutex::new(ConditionalOperationRoutes {
                records,
                whole_graph,
                bootstrap,
            })),
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

    pub(in crate::domain_computation::primary_graph) fn publish_routes(
        &self,
        operation: &dyn WorthQueryInstalledConditionalOperation<Schema>,
    ) -> bool {
        let (records, whole_graph) = operation.authoritative_commit_routes();
        let bootstrap = operation.bootstrap_commit_route_pending();
        let mut routes = self
            .routes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if routes.records == records
            && routes.whole_graph == whole_graph
            && routes.bootstrap == bootstrap
        {
            return false;
        }
        *routes = ConditionalOperationRoutes {
            records,
            whole_graph,
            bootstrap,
        };
        true
    }

    pub(super) fn commit_routes(&self) -> (Vec<RecordRef>, bool, bool) {
        let routes = self
            .routes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        (routes.records.clone(), routes.whole_graph, routes.bootstrap)
    }

    pub(in crate::domain_computation::primary_graph::conditional_operation) fn observe_clock(
        &self,
        bridge: &BridgeSealedRuntimeAssembly,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        truth: &WorthQueryConditionalTruthBasis,
    ) -> WorthQueryObservedConditionalOperation {
        let mut operation = self.lock_operation();
        if let Err(denial) = operation.select_product_binding(bridge, runtime, truth) {
            return WorthQueryObservedConditionalOperation {
                outcome: ErasedClockObservationOutcome::Failed {
                    kind: installation_denial_observation_kind(denial.kind()),
                    detail: denial.subject().to_string(),
                },
                routes_changed: false,
            };
        }
        let outcome = operation.observe_clock(bridge, runtime, truth);
        let routes_changed = self.publish_routes(operation.as_ref());
        WorthQueryObservedConditionalOperation {
            outcome,
            routes_changed,
        }
    }

    pub(in crate::domain_computation::primary_graph::conditional_operation) fn admit_product_binding(
        &self,
        bridge: &BridgeSealedRuntimeAssembly,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        truth: &WorthQueryConditionalTruthBasis,
    ) -> Result<bool, super::super::installation::WorthQueryConditionalRuntimeInstallationDenial>
    {
        let mut operation = self.lock_operation();
        operation.select_product_binding(bridge, runtime, truth)?;
        Ok(self.publish_routes(operation.as_ref()))
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
