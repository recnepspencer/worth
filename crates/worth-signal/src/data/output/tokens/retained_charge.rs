use super::{ArtifactContinuityToken, OutputIdentity};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for OutputIdentity {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            value,
            stable_hash: _,
        } = self;
        value.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for ArtifactContinuityToken {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            value,
            stable_hash: _,
        } = self;
        value.retained_heap_charge(work)
    }
}
