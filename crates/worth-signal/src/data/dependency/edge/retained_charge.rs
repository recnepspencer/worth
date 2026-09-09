use super::{DependencyEdge, DependencySortKey};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for DependencyEdge {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            source: _,
            aspect: _,
            scope,
            interned_scope: _,
        } = self;
        scope.retained_heap_charge(work)
    }
}
impl RetainedStorageMeasurement for DependencySortKey {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            source_index: _,
            source_generation: _,
            aspect_index: _,
            scope,
        } = self;
        scope.retained_heap_charge(work)
    }
}
