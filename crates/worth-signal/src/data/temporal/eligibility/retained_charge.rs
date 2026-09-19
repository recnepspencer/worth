use super::TemporalExecutionSummary;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for TemporalExecutionSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            ready_count: _,
            deferred_count: _,
            runtime_clock_authority_count: _,
            resolver_fallback_count: _,
            runtime_scheduled_wake_count: _,
        } = self;
        Ok(Charge::ZERO)
    }
}
