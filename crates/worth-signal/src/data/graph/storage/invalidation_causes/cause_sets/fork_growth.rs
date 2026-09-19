use super::CanonicalCauseSetStore;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageForkGrowth,
    RetainedStorageForkGrowthDenial as Denial, RetainedStoragePreparation as Work,
};

impl RetainedStorageForkGrowth for CanonicalCauseSetStore {
    fn prepare_fork_growth(&mut self, work: &mut Work) -> Result<Charge, Denial> {
        work.reserve_visits(std::mem::size_of::<Self>())?;
        let Self {
            sets,
            slot_generations,
            free_indices,
            published_output_commits,
            output_commit_reference_counts,
            retained_custody: _,
            generation: _,
            next_output_commit_ordinal: _,
            occupied_set_count: _,
            deserialized_quarantine: _,
            #[cfg(test)]
                published_order_probe: _,
            #[cfg(test)]
                last_compaction_slot_visits: _,
        } = self;
        let mut growth = Charge::ZERO;
        macro_rules! add {
            ($root:expr) => {
                growth = growth.checked_add($root.prepare_fork_growth(work)?)?;
            };
        }
        add!(sets);
        add!(slot_generations);
        add!(free_indices);
        add!(published_output_commits);
        add!(output_commit_reference_counts);
        Ok(growth)
    }
}
