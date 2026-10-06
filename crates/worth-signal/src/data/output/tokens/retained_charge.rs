use super::{
    ArtifactContinuityToken, ComputationFamily, ComputationKey, OutputIdentity, StructuralMemoKey,
};
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

macro_rules! measured_computation_token {
    ($($token:ty),+ $(,)?) => {$(
        impl RetainedStorageMeasurement for $token {
            fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
                work.visit()?;
                self.value.retained_heap_charge(work)
            }
        }
    )+};
}

measured_computation_token!(ComputationFamily, ComputationKey, StructuralMemoKey);
