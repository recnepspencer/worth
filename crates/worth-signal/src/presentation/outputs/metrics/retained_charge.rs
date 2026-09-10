use super::GraphMetrics;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for GraphMetrics {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            evaluation: _,
            invalidation: _,
            planner: _,
            execution: _,
            storage: _,
            temporal: _,
            host_computed: _,
            partition_interner_size: _,
        } = self;
        Ok(Charge::ZERO)
    }
}
