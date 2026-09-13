use super::EdgeTopology;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for EdgeTopology {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            dependency_snapshots,
            dependency_snapshot_shapes,
            dependency_snapshot_storage_custody: _,
            dependency_edges,
            subscriber_edges,
            reverse_subscriptions,
            pending_revalidation_waiters,
            pending_revalidation_storage_custody: _,
        } = self;
        Ok(Charge::ZERO
            .checked_add(dependency_snapshots.retained_heap_charge(work)?)?
            .checked_add(dependency_snapshot_shapes.retained_heap_charge(work)?)?
            .checked_add(dependency_edges.retained_heap_charge(work)?)?
            .checked_add(subscriber_edges.retained_heap_charge(work)?)?
            .checked_add(reverse_subscriptions.retained_heap_charge(work)?)?
            .checked_add(pending_revalidation_waiters.retained_heap_charge(work)?)?)
    }
}

use crate::data::retained_storage::{RetainedStorageForkCharge, RetainedStorageForkPreparation};

impl RetainedStorageForkPreparation for EdgeTopology {
    fn prepare_fork_charge(
        &mut self,
        work: &mut Preparation,
    ) -> Result<RetainedStorageForkCharge, Denial> {
        work.visit()?;
        let Self {
            dependency_snapshots,
            dependency_snapshot_shapes,
            dependency_snapshot_storage_custody: _,
            dependency_edges,
            subscriber_edges,
            reverse_subscriptions,
            pending_revalidation_waiters,
            pending_revalidation_storage_custody: _,
        } = self;
        Ok(RetainedStorageForkCharge::unchanged(Charge::ZERO)
            .checked_add(dependency_snapshots.prepare_fork_charge(work)?)?
            .checked_add(dependency_snapshot_shapes.prepare_fork_charge(work)?)?
            .checked_add(dependency_edges.prepare_fork_charge(work)?)?
            .checked_add(subscriber_edges.prepare_fork_charge(work)?)?
            .checked_add(reverse_subscriptions.prepare_fork_charge(work)?)?
            .checked_add(pending_revalidation_waiters.prepare_fork_charge(work)?)?)
    }
}
