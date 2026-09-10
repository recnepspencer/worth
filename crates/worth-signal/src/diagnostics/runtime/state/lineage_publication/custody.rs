use crate::data::retained_storage::{
    RetainedStorageCharge, RetainedStorageMeasurement, RetainedStoragePreparation,
    RetainedStoragePreparationDenial, SignalConditionalRetentionReservation,
};
use std::sync::Arc;

/// Accounting follows retained diagnostics, but is not diagnostic meaning.
#[derive(Debug, Clone, Default)]
pub(crate) struct LineageRetentionCustody(
    pub(super) Option<Arc<SignalConditionalRetentionReservation>>,
);
impl PartialEq for LineageRetentionCustody {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl RetainedStorageMeasurement for LineageRetentionCustody {
    fn retained_heap_charge(
        &self,
        work: &mut RetainedStoragePreparation,
    ) -> Result<RetainedStorageCharge, RetainedStoragePreparationDenial> {
        self.0.retained_heap_charge(work)
    }
}
