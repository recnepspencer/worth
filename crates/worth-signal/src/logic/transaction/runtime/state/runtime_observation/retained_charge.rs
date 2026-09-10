use super::ObservedNodeSet;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for ObservedNodeSet {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { nodes } = self;
        nodes.retained_heap_charge(work)
    }
}
