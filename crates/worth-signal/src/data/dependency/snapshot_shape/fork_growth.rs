use super::DependencySnapshotShapeStore;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageForkGrowth,
    RetainedStorageForkGrowthDenial as Denial, RetainedStoragePreparation as Work,
};
impl RetainedStorageForkGrowth for DependencySnapshotShapeStore {
    fn prepare_fork_growth(&mut self, work: &mut Work) -> Result<Charge, Denial> {
        work.reserve_visits(std::mem::size_of::<Self>())?;
        let Self { shapes, interner } = self;
        let mut growth = Charge::ZERO;
        growth = growth.checked_add(shapes.prepare_fork_growth(work)?)?;
        growth = growth.checked_add(interner.prepare_fork_growth(work)?)?;
        Ok(growth)
    }
}

impl DependencySnapshotShapeStore {
    pub(crate) fn epoch_fork_growth_bound(&self) -> Result<u64, crate::data::error::SignalError> {
        let shapes = self
            .shapes
            .prepared_fork_charge()
            .map_err(|_| crate::data::error::SignalError::SnapshotIndexUnavailable)?;
        let interner = self
            .interner
            .prepared_fork_charge()
            .map_err(|_| crate::data::error::SignalError::SnapshotIndexUnavailable)?;
        shapes
            .source_growth
            .checked_add(interner.source_growth)
            .map(Charge::bytes)
            .map_err(|_| crate::data::error::SignalError::EvaluationStorageCapacityExhausted)
    }
}
