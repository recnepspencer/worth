use super::{CausalityCarryPolicy, RetainedArtifactCarryPolicy, RuntimeArtifactCarryPolicy};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for RuntimeArtifactCarryPolicy {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::CarryMergeAdoptable | Self::RebuildAfterAdoption | Self::DoNotCarry => {
                Ok(Charge::ZERO)
            }
        }
    }
}
impl RetainedStorageMeasurement for RetainedArtifactCarryPolicy {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::CarryIfPolicyAllows | Self::ReconstructIfNeeded | Self::Drop => Ok(Charge::ZERO),
        }
    }
}
impl RetainedStorageMeasurement for CausalityCarryPolicy {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::CarryIfPolicyAllows | Self::Drop => Ok(Charge::ZERO),
        }
    }
}
