//! Whole-Source expectation retains owner-issued material, never an Execution loan.
use std::sync::Arc;
use worth_relational::facade::{
    identity::EntityId,
    runtime::{ManagedIndexSourceObservation, RelationalRuntime},
    snapshots::SnapshotHandle,
};

/// Descriptive locator within the already checked schema/runtime/branch affinity.
/// The retained Native observation, not this key, supplies Source currentness.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct ManagedSourceReadKey {
    pub obligation: [u8; 32],
    pub scope: EntityId,
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct ManagedSourceRead {
    key: ManagedSourceReadKey,
    native: Arc<ManagedIndexSourceObservation>,
    _allocation: Arc<crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation>,
}
impl ManagedSourceRead {
    pub(in crate::domain_computation::primary_graph) fn new(
        obligation: [u8; 32],
        scope: EntityId,
        native: Arc<ManagedIndexSourceObservation>,
        allocation: Arc<
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
        >,
    ) -> Self {
        Self {
            key: ManagedSourceReadKey { obligation, scope },
            native,
            _allocation: allocation,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn key(&self) -> &ManagedSourceReadKey {
        &self.key
    }
    pub(in crate::domain_computation::primary_graph) fn native(
        &self,
    ) -> &ManagedIndexSourceObservation {
        &self.native
    }
    pub(in crate::domain_computation::primary_graph) fn remains_equal(
        &self,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
    ) -> Result<bool, super::WorthQuerySourceCurrentnessFailure> {
        use super::WorthQuerySourceCurrentnessFailure as Stop;
        runtime
            .read_truth()
            .managed_index_source_is_current(execution, snapshot, &self.native)
            .map_err(|cause| match cause {
                worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial::Source(
                    worth_relational::facade::runtime::DerivedRelationReadDenial::Request(cause),
                ) => Stop::Execution(cause),
                _ => Stop::Unavailable,
            })
    }
}
impl PartialEq for ManagedSourceRead {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.native.same_source(&other.native)
    }
}
impl Eq for ManagedSourceRead {}
impl std::fmt::Debug for ManagedSourceRead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManagedSourceRead")
            .field("key", &self.key)
            .field("revision", &self.native.source_revision())
            .finish_non_exhaustive()
    }
}
