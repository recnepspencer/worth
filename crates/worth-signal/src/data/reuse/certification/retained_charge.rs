use super::{ReuseBoundaryProof, ReuseCertificationRecord};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for ReuseCertificationRecord {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            strategy: _,
            origin: _,
            source: _,
            crossing: _,
            proofs,
        } = self;
        proofs.retained_heap_charge(work)
    }
}
impl RetainedStorageMeasurement for ReuseBoundaryProof {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            boundary: _,
            satisfied: _,
        } = self;
        Ok(Charge::ZERO)
    }
}
