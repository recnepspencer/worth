use super::PartitionInterner;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageForkGrowth,
    RetainedStorageForkGrowthDenial as Denial, RetainedStoragePreparation as Work,
};

impl RetainedStorageForkGrowth for PartitionInterner {
    fn prepare_fork_growth(&mut self, work: &mut Work) -> Result<Charge, Denial> {
        work.reserve_visits(std::mem::size_of::<Self>())?;
        let Self {
            segments,
            segment_lookup,
        } = self;
        let mut growth = Charge::ZERO;
        growth = growth.checked_add(segments.prepare_fork_growth(work)?)?;
        growth = growth.checked_add(segment_lookup.prepare_fork_growth(work)?)?;
        Ok(growth)
    }
}
