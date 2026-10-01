use super::NodeId;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for NodeId {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            index: _,
            generation: _,
        } = self;
        Ok(Charge::ZERO)
    }
}
impl worth_execution::ChargedBytes for super::NodeId {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
