//! Reservation handles retain accounting authority, not another payload tree.
use super::SignalConditionalRetentionReservation;
use crate::data::retained_storage::{
    RetainedStorageCharge, RetainedStorageMeasurement, RetainedStoragePreparation,
    RetainedStoragePreparationDenial,
};

impl RetainedStorageMeasurement for SignalConditionalRetentionReservation {
    fn retained_heap_charge(
        &self,
        work: &mut RetainedStoragePreparation,
    ) -> Result<RetainedStorageCharge, RetainedStoragePreparationDenial> {
        work.visit()?;
        // The owner ledger is a shared accounting service, not payload storage
        // reachable through this reservation. Arc/Option owners count the
        // reservation allocation and its inline handle at their own boundary.
        Ok(RetainedStorageCharge::ZERO)
    }
}
