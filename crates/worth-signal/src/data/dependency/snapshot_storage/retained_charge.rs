use super::{DependencySnapshotId, DependencySnapshotStore};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for DependencySnapshotId {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self(_) = self;
        Ok(Charge::ZERO)
    }
}
impl RetainedStorageMeasurement for DependencySnapshotStore {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            snapshots,
            interner,
            shape_handles,
        } = self;
        snapshots
            .retained_heap_charge(work)?
            .checked_add(interner.retained_heap_charge(work)?)?
            .checked_add(shape_handles.retained_heap_charge(work)?)
    }
}

use crate::data::retained_storage::{RetainedStorageForkCharge, RetainedStorageForkPreparation};

impl RetainedStorageForkPreparation for DependencySnapshotStore {
    fn prepare_fork_charge(
        &mut self,
        work: &mut Preparation,
    ) -> Result<RetainedStorageForkCharge, Denial> {
        work.visit()?;
        let Self {
            snapshots,
            interner,
            shape_handles,
        } = self;
        snapshots
            .prepare_fork_charge(work)?
            .checked_add(interner.prepare_fork_charge(work)?)?
            .checked_add(shape_handles.prepare_fork_charge(work)?)
    }
}
