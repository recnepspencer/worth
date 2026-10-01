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

impl DependencySnapshotStore {
    pub(crate) fn epoch_fork_growth_bound(&self) -> Result<u64, crate::data::error::SignalError> {
        let snapshots = self
            .snapshots
            .prepared_fork_charge()
            .map_err(|_| crate::data::error::SignalError::SnapshotIndexUnavailable)?;
        let interner = self
            .interner
            .prepared_fork_charge()
            .map_err(|_| crate::data::error::SignalError::SnapshotIndexUnavailable)?;
        let handles = self
            .shape_handles
            .prepared_fork_charge()
            .map_err(|_| crate::data::error::SignalError::SnapshotIndexUnavailable)?;
        snapshots
            .source_growth
            .checked_add(interner.source_growth)
            .and_then(|charge| charge.checked_add(handles.source_growth))
            .map(Charge::bytes)
            .map_err(|_| crate::data::error::SignalError::EvaluationStorageCapacityExhausted)
    }
}
