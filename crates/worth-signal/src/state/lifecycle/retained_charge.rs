use super::{SignalBranchHandle, SignalBranchId};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for SignalBranchId {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self(_) = self;
        Ok(Charge::ZERO)
    }
}

impl RetainedStorageMeasurement for SignalBranchHandle {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            id: _,
            name,
            parent_branch_id: _,
            head_snapshot_id: _,
        } = self;
        name.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for super::SnapshotRestoreIntent {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            state: _,
            artifacts: _,
            dependency_state: _,
        } = self;
        Ok(Charge::ZERO)
    }
}

impl RetainedStorageMeasurement for super::SignalSnapshotId {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self(_) = self;
        Ok(Charge::ZERO)
    }
}
