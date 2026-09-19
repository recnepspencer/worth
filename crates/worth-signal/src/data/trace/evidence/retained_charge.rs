use super::CausalityMetadata;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for CausalityMetadata {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { kind, fields } = self;
        kind.retained_heap_charge(work)?
            .checked_add(fields.retained_heap_charge(work)?)
    }
}
