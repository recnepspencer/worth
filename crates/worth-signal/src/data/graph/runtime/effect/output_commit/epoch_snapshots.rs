//! One cumulative snapshot and shape-store draft for an output epoch.
use std::sync::Arc;

use super::snapshot_preparation::PreparedSnapshotCandidate;
use super::{EvaluationWork, SignalError, SignalGraph};
use crate::data::dependency::{
    DependencySnapshotId, DependencySnapshotShapeStore, DependencySnapshotStore,
    SnapshotDeltaRecord, SnapshotStorageStrategy,
};
use crate::data::handle::NodeId;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageCharge as Charge, RetainedStorageForkGrowth,
    RetainedStoragePreparation as Work, SignalConditionalRetentionReservation as Reservation,
};

pub(super) struct PreparedEpochSnapshotStore {
    snapshots: DependencySnapshotStore,
    shapes: DependencySnapshotShapeStore,
    updates: Vec<(
        NodeId,
        DependencySnapshotId,
        SnapshotDeltaRecord,
        SnapshotStorageStrategy,
    )>,
    previous: Option<Arc<Reservation>>,
    resources: Option<Reservation>,
}

impl SignalGraph {
    pub(crate) fn epoch_snapshot_store_fork_growth_bound(&self) -> Result<u64, SignalError> {
        self.topology
            .dependency_snapshots
            .epoch_fork_growth_bound()?
            .checked_add(
                self.topology
                    .dependency_snapshot_shapes
                    .epoch_fork_growth_bound()?,
            )
            .ok_or_else(|| SignalError::invalid_input("snapshot fork growth overflow"))
    }

    pub(super) fn prepare_epoch_snapshot_store(
        &mut self,
        candidates: Vec<PreparedSnapshotCandidate>,
        mut preparation: Option<&mut SignalPreparationBudget>,
        retained_work: &mut Work<'_>,
    ) -> Result<Option<PreparedEpochSnapshotStore>, SignalError> {
        if candidates.is_empty() {
            return Ok(None);
        }
        if let Some(budget) = preparation.as_deref_mut() {
            budget.claim_vec::<(
                NodeId,
                DependencySnapshotId,
                SnapshotDeltaRecord,
                SnapshotStorageStrategy,
            )>(candidates.len())?;
            budget.claim_vec::<DependencySnapshotStore>(1)?;
            budget.claim_vec::<DependencySnapshotShapeStore>(1)?;
        }
        let fork_growth = self
            .topology
            .dependency_snapshots
            .prepare_fork_growth(retained_work)
            .map_err(map_fork)?
            .checked_add(
                self.topology
                    .dependency_snapshot_shapes
                    .prepare_fork_growth(retained_work)
                    .map_err(map_fork)?,
            )
            .map_err(map_charge)?;
        if let Some(budget) = preparation {
            budget.claim(fork_growth.bytes())?;
        }
        let retained = self.arena.retained_node_ledger.clone();
        let (mut snapshots, mut shapes, previous, mut resources) = if let Some(ledger) = retained {
            let growth = fork_growth
                .checked_add(Charge::capacity::<PreparedEpochSnapshotStore>(1).map_err(map_charge)?)
                .and_then(|charge| charge.checked_add(arc_allocation_charge::<Reservation>()?))
                .map_err(map_charge)?;
            let mut resources = ledger.reserve(0, growth).map_err(map_retention)?;
            let snapshots = self
                .topology
                .dependency_snapshots
                .fork_reserved(&mut resources);
            let shapes = self
                .topology
                .dependency_snapshot_shapes
                .fork_reserved(&mut resources);
            (
                snapshots,
                shapes,
                self.topology.dependency_snapshot_storage_custody.clone(),
                Some(resources),
            )
        } else {
            (
                self.topology.dependency_snapshots.fork_persistent(),
                self.topology.dependency_snapshot_shapes.fork_persistent(),
                None,
                None,
            )
        };
        let mut updates = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            let insertion = snapshots.prepare_insertion(
                candidate.snapshot,
                &mut shapes,
                &mut EvaluationWork::Conditional(retained_work),
            )?;
            let id = insertion.snapshot_id();
            if let Some(resources) = resources.as_mut() {
                let staging =
                    insertion.retained_staging_charge(&snapshots, &shapes, retained_work)?;
                resources.grow(staging).map_err(map_retention)?;
                insertion.publish_retained(&mut snapshots, &mut shapes, retained_work)?;
            } else {
                insertion.publish(&mut snapshots, &mut shapes);
            }
            updates.push((candidate.node, id, candidate.delta, candidate.strategy));
        }
        if let Some(resources) = resources.as_mut() {
            let final_payload = snapshots
                .prepared_retained_charge()?
                .checked_add(shapes.prepared_retained_charge()?)
                .map_err(map_charge)?;
            resources
                .shrink_payload_to(final_payload)
                .map_err(map_retention)?;
        }
        Ok(Some(PreparedEpochSnapshotStore {
            snapshots,
            shapes,
            updates,
            previous,
            resources,
        }))
    }
}

impl PreparedEpochSnapshotStore {
    pub(super) fn snapshot_id(&self, node: NodeId) -> Option<DependencySnapshotId> {
        self.updates
            .iter()
            .find(|(target, ..)| *target == node)
            .map(|(_, id, ..)| *id)
    }

    pub(super) fn publish(self, graph: &mut SignalGraph) {
        let old_snapshots =
            std::mem::replace(&mut graph.topology.dependency_snapshots, self.snapshots);
        let old_shapes =
            std::mem::replace(&mut graph.topology.dependency_snapshot_shapes, self.shapes);
        if let Some(resources) = self.resources {
            graph.topology.dependency_snapshot_storage_custody = Some(Arc::new(resources));
        }
        drop(old_snapshots);
        drop(old_shapes);
        drop(self.previous);
        for (node, _, delta, strategy) in self.updates {
            graph.record_dependency_snapshot_storage_publication(node, delta, strategy);
        }
    }
}

fn map_fork(_: crate::data::retained_storage::RetainedStorageForkGrowthDenial) -> SignalError {
    SignalError::SnapshotIndexUnavailable
}
fn map_charge(_: crate::data::retained_storage::RetainedStoragePreparationDenial) -> SignalError {
    SignalError::EvaluationStorageCapacityExhausted
}
fn map_retention(
    _: crate::data::retained_storage::SignalConditionalRetentionDenial,
) -> SignalError {
    SignalError::EvaluationStorageCapacityExhausted
}
