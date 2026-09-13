use super::TaskReason;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for TaskReason {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::Dirty
            | Self::MaybeStaleValidation
            | Self::ConditionForced
            | Self::RequestedTarget
            | Self::DependencyRequired
            | Self::MemoValidation
            | Self::PartitionScopedDependency
            | Self::OutputDiffDependent => Ok(Charge::ZERO),
        }
    }
}
