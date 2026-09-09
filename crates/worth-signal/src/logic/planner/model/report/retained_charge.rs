use super::{StageExecutionOutcome, TaskExecutionOutcome};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for TaskExecutionOutcome {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::Recomputed
            | Self::ValidatedClean
            | Self::ConditionDeferred
            | Self::ConditionRevertedClean
            | Self::MemoizedReuse
            | Self::SnapshotRestoreReuse
            | Self::ReconciliationAdoption
            | Self::CrossIdentityPersistentReuse
            | Self::PartialArtifactSplice
            | Self::PropagationSuppressed
            | Self::Pruned => Ok(Charge::ZERO),
        }
    }
}
impl RetainedStorageMeasurement for StageExecutionOutcome {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::CompletedSerial => Ok(Charge::ZERO),
            #[cfg(feature = "parallel")]
            Self::CompletedParallel => Ok(Charge::ZERO),
        }
    }
}
