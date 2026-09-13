use super::{DependencySnapshot, DependencySnapshotEntry};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for DependencySnapshot {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { entries } = self;
        entries.retained_heap_charge(work)
    }
}
impl RetainedStorageMeasurement for DependencySnapshotEntry {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            source: _,
            aspect: _,
            cached_version: _,
            scope,
        } = self;
        scope.retained_heap_charge(work)
    }
}
