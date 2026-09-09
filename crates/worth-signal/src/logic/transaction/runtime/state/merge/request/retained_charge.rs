use super::SignalSelectedAspectRequestEntry;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for SignalSelectedAspectRequestEntry {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { node: _, aspect: _ } = self;
        Ok(Charge::ZERO)
    }
}
