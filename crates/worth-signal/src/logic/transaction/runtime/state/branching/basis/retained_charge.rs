use super::{SignalBranchHeadPosture, SignalBranchRestorePosture};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for SignalBranchHeadPosture {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::NoHeadSnapshot | Self::HeadSnapshot(_) => Ok(Charge::ZERO),
        }
    }
}
impl RetainedStorageMeasurement for SignalBranchRestorePosture {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::NotRestoreDerived => Ok(Charge::ZERO),
            Self::SnapshotRestore {
                snapshot_id: _,
                intent,
            } => intent.retained_heap_charge(work),
        }
    }
}
