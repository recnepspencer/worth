use super::PartitionInterner;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageForkGrowth,
    RetainedStorageForkGrowthDenial as Denial, RetainedStoragePreparation as Work,
};

impl RetainedStorageForkGrowth for PartitionInterner {
    fn prepare_fork_growth(&mut self, work: &mut Work) -> Result<Charge, Denial> {
        work.reserve_visits(std::mem::size_of::<Self>())?;
        let Self {
            partitions,
            details,
            partition_lookup,
            detail_lookup,
        } = self;
        let mut growth = Charge::ZERO;
        growth = growth.checked_add(partitions.prepare_fork_growth(work)?)?;
        growth = growth.checked_add(details.prepare_fork_growth(work)?)?;
        growth = growth.checked_add(partition_lookup.prepare_fork_growth(work)?)?;
        growth = growth.checked_add(detail_lookup.prepare_fork_growth(work)?)?;
        Ok(growth)
    }
}
