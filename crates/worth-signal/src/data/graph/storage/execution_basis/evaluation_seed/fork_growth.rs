use super::SignalEvaluationStorage;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageForkGrowth,
    RetainedStorageForkGrowthDenial as Denial, RetainedStoragePreparation as Work,
};
impl RetainedStorageForkGrowth for SignalEvaluationStorage {
    fn prepare_fork_growth(&mut self, work: &mut Work) -> Result<Charge, Denial> {
        work.reserve_visits(std::mem::size_of::<Self>())?;
        let Self {
            hot,
            warm,
            cold,
            topology,
            causes,
            conditional_versions,
            conditional_versions_custody: _,
            repeated_admissions,
            partitions,
            diagnostics,
            retained_node_ledger: _,
            retained_node_custody: _,
            retained_seed_custody: _,
            fork_custody: _,
            compaction: _,
            cause_readmission_required: _,
        } = self;
        let mut growth = Charge::ZERO;
        macro_rules! add {
            ($root:expr) => {
                growth = growth.checked_add($root.prepare_fork_growth(work)?)?
            };
        }
        add!(hot);
        add!(warm);
        add!(cold);
        add!(topology);
        add!(causes);
        add!(conditional_versions);
        add!(repeated_admissions);
        add!(partitions);
        add!(diagnostics);
        Ok(growth)
    }
}

#[cfg(test)]
mod tests;
