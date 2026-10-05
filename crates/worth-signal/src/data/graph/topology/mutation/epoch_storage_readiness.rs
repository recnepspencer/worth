//! Cold segment index readiness and pre-dispatch insertion ceilings.
use worth_execution::MapKernelContext;

use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::retained_storage::RetainedStorageForkPreparation;

impl SignalGraph {
    pub(crate) fn epoch_dependency_set_capacity_bound(
        &self,
        desired_edges: usize,
        desired_scope_heap: u64,
    ) -> Result<u64, SignalError> {
        Ok(self
            .topology
            .dependency_edges
            .insertion_payload_capacity_bound(desired_edges, desired_scope_heap)?
            .bytes())
    }

    pub(crate) fn epoch_dependency_batch_structure_capacity_bound(
        &self,
        target_count: usize,
    ) -> Result<u64, SignalError> {
        Ok(self
            .topology
            .dependency_edges
            .batch_structure_capacity_bound(target_count)?
            .bytes())
    }

    pub(crate) fn prepare_epoch_topology_storage_readiness(
        &mut self,
        request: Option<&mut MapKernelContext<'_, '_>>,
        mut budget: Option<&mut SignalPreparationBudget>,
    ) -> Result<(), SignalError> {
        super::epoch_preparation::with_segment_work(request, |observed| {
            self.cause_sets
                .prepare_fork_charge(observed)
                .map_err(|_| SignalError::EvaluationStorageUnavailable)?;
            self.topology
                .dependency_snapshots
                .prepare_fork_charge(observed)
                .map_err(|_| SignalError::SnapshotIndexUnavailable)?;
            self.topology
                .dependency_snapshot_shapes
                .prepare_fork_charge(observed)
                .map_err(|_| SignalError::SnapshotIndexUnavailable)?;
            self.topology
                .dependency_edges
                .ensure_interner_ready(observed, budget.as_deref_mut())?;
            self.topology
                .subscriber_edges
                .ensure_interner_ready(observed, budget)
        })
    }
}
