//! Atomic retained publication of dependency snapshot and shape storage.
use super::{EvaluationWork, SignalError, SignalGraph};
use crate::data::dependency::{
    DependencySnapshotId, DependencySnapshotShapeStore, DependencySnapshotStore,
    PreparedSnapshotInsertion, SnapshotDeltaRecord, SnapshotStorageStrategy,
};
use crate::data::graph::runtime::graph::{map_node_edit_accounting, map_node_edit_retention};
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageCharge as Charge, RetainedStorageForkGrowth,
    RetainedStorageForkGrowthDenial, RetainedStoragePreparation as Work,
    SignalConditionalRetentionReservation as Reservation,
};
use std::sync::Arc;

#[derive(Debug)]
pub(super) enum PreparedEffectSnapshotStorage {
    Ordinary(PreparedSnapshotInsertion),
    Retained(PreparedRetainedSnapshotStorage),
}

#[derive(Debug)]
pub(super) struct PreparedRetainedSnapshotStorage {
    id: DependencySnapshotId,
    snapshots: DependencySnapshotStore,
    shapes: DependencySnapshotShapeStore,
    // Keep prior accounting live until both prior payload roots are dropped.
    previous: Option<Arc<Reservation>>,
    resources: Reservation,
}

impl PreparedEffectSnapshotStorage {
    pub(super) fn snapshot_id(&self) -> DependencySnapshotId {
        match self {
            Self::Ordinary(insertion) => insertion.snapshot_id(),
            Self::Retained(publication) => publication.id,
        }
    }
}

impl SignalGraph {
    pub(super) fn prepare_effect_snapshot_storage(
        &mut self,
        insertion: PreparedSnapshotInsertion,
        allowance: &mut EvaluationWork<'_>,
    ) -> Result<PreparedEffectSnapshotStorage, SignalError> {
        if self.arena.retained_node_ledger.is_none() || !insertion.changes_storage() {
            return Ok(PreparedEffectSnapshotStorage::Ordinary(insertion));
        }
        match allowance {
            EvaluationWork::Conditional(work) => self
                .prepare_retained_snapshot_storage(insertion, work)
                .map(PreparedEffectSnapshotStorage::Retained),
            EvaluationWork::Ordinary => {
                let maximum = self
                    .installed_runtime_policy()
                    .conditional_evaluation_budget()
                    .maximum_attempt_visits;
                self.prepare_retained_snapshot_storage(insertion, &mut Work::new(maximum))
                    .map(PreparedEffectSnapshotStorage::Retained)
            }
        }
    }

    fn prepare_retained_snapshot_storage(
        &mut self,
        insertion: PreparedSnapshotInsertion,
        work: &mut Work,
    ) -> Result<PreparedRetainedSnapshotStorage, SignalError> {
        let ledger = self
            .arena
            .retained_node_ledger
            .as_ref()
            .expect("retained snapshot publication has a ledger")
            .clone();
        let maximum = insertion.retained_staging_charge(
            &self.topology.dependency_snapshots,
            &self.topology.dependency_snapshot_shapes,
            work,
        )?;
        let source_growth = self
            .topology
            .dependency_snapshots
            .prepare_fork_growth(work)
            .map_err(map_fork_denial)?
            .checked_add(
                self.topology
                    .dependency_snapshot_shapes
                    .prepare_fork_growth(work)
                    .map_err(map_fork_denial)?,
            )
            .map_err(map_node_edit_accounting)?;
        let total = maximum
            .checked_add(source_growth)
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<PreparedRetainedSnapshotStorage>(1)?)
            })
            .and_then(|charge| charge.checked_add(arc_allocation_charge::<Reservation>()?))
            .map_err(map_node_edit_accounting)?;
        work.reserve_visits(std::mem::size_of::<PreparedRetainedSnapshotStorage>())
            .map_err(map_node_edit_accounting)?;
        let mut resources = ledger.reserve(0, total).map_err(map_node_edit_retention)?;
        let mut snapshots = self
            .topology
            .dependency_snapshots
            .fork_reserved(&mut resources);
        let mut shapes = self
            .topology
            .dependency_snapshot_shapes
            .fork_reserved(&mut resources);
        let previous = self.topology.dependency_snapshot_storage_custody.clone();
        let (id, _) = insertion.publish_retained(&mut snapshots, &mut shapes, work)?;
        let retained = snapshots
            .prepared_retained_charge()?
            .checked_add(shapes.prepared_retained_charge()?)
            .map_err(map_node_edit_accounting)?;
        if retained > maximum {
            return Err(SignalError::EvaluationStorageCapacityExhausted);
        }
        let final_payload = retained
            .checked_add(arc_allocation_charge::<Reservation>().map_err(map_node_edit_accounting)?)
            .map_err(map_node_edit_accounting)?;
        resources
            .shrink_payload_to(final_payload)
            .map_err(map_node_edit_retention)?;
        Ok(PreparedRetainedSnapshotStorage {
            id,
            snapshots,
            shapes,
            previous,
            resources,
        })
    }

    pub(super) fn publish_effect_snapshot_storage(
        &mut self,
        node: crate::data::handle::NodeId,
        publication: PreparedEffectSnapshotStorage,
        delta: SnapshotDeltaRecord,
        strategy: SnapshotStorageStrategy,
    ) {
        match publication {
            PreparedEffectSnapshotStorage::Ordinary(insertion) => {
                self.publish_dependency_snapshot_storage(node, insertion, delta, strategy);
            }
            PreparedEffectSnapshotStorage::Retained(publication) => {
                let PreparedRetainedSnapshotStorage {
                    id: _,
                    snapshots,
                    shapes,
                    previous,
                    resources,
                } = publication;
                let old_snapshots =
                    std::mem::replace(&mut self.topology.dependency_snapshots, snapshots);
                let old_shapes =
                    std::mem::replace(&mut self.topology.dependency_snapshot_shapes, shapes);
                self.topology.dependency_snapshot_storage_custody = Some(Arc::new(resources));
                drop(old_snapshots);
                drop(old_shapes);
                drop(previous);
                self.record_dependency_snapshot_storage_publication(node, delta, strategy);
            }
        }
    }
}

fn map_fork_denial(denial: RetainedStorageForkGrowthDenial) -> SignalError {
    match denial {
        RetainedStorageForkGrowthDenial::Accounting(denial) => map_node_edit_accounting(denial),
        RetainedStorageForkGrowthDenial::PreparationRequired => {
            SignalError::SnapshotIndexUnavailable
        }
    }
}
