use super::DependencySnapshotStore;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageForkGrowth,
    RetainedStorageForkGrowthDenial as Denial, RetainedStoragePreparation as Work,
};
impl RetainedStorageForkGrowth for DependencySnapshotStore {
    fn prepare_fork_growth(&mut self, work: &mut Work) -> Result<Charge, Denial> {
        work.reserve_visits(std::mem::size_of::<Self>())?;
        let Self {
            snapshots,
            interner,
            shape_handles,
        } = self;
        let mut growth = Charge::ZERO;
        growth = growth.checked_add(snapshots.prepare_fork_growth(work)?)?;
        growth = growth.checked_add(interner.prepare_fork_growth(work)?)?;
        growth = growth.checked_add(shape_handles.prepare_fork_growth(work)?)?;
        Ok(growth)
    }
}
