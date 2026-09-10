use super::{DependencySnapshotShape, DependencySnapshotShapeStore, SnapshotShapeHandle};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for SnapshotShapeHandle {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self(_) = self;
        Ok(Charge::ZERO)
    }
}
impl RetainedStorageMeasurement for DependencySnapshotShape {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { keys } = self;
        keys.retained_heap_charge(work)
    }
}
impl RetainedStorageMeasurement for DependencySnapshotShapeStore {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { shapes, interner } = self;
        shapes
            .retained_heap_charge(work)?
            .checked_add(interner.retained_heap_charge(work)?)
    }
}

use crate::data::retained_storage::{RetainedStorageForkCharge, RetainedStorageForkPreparation};

impl RetainedStorageForkPreparation for DependencySnapshotShapeStore {
    fn prepare_fork_charge(
        &mut self,
        work: &mut Preparation,
    ) -> Result<RetainedStorageForkCharge, Denial> {
        work.visit()?;
        let Self { shapes, interner } = self;
        shapes
            .prepare_fork_charge(work)?
            .checked_add(interner.prepare_fork_charge(work)?)
    }
}
