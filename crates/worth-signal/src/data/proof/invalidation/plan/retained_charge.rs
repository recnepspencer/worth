use super::InvalidationPlanningEstimate;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for InvalidationPlanningEstimate {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            seed_count: _,
            group_count: _,
            direct_wave_count: _,
            transitive_wave_count: _,
            direct_dirty_count: _,
            maybe_stale_count: _,
            partition_scoped_checks: _,
            partition_match_count: _,
            detail_match_count: _,
            cycle_check_candidate_count: _,
        } = self;
        Ok(Charge::ZERO)
    }
}
