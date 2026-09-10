use super::EvaluationPlanSummary;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for EvaluationPlanSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            profile: _,
            requested_target_count: _,
            stage_count: _,
            task_count: _,
            max_stage_width: _,
            contract_pruned_count: _,
            stage_widths,
            direct_request_count: _,
            transitive_task_count: _,
            task_reason_counts,
        } = self;
        stage_widths
            .retained_heap_charge(work)?
            .checked_add(task_reason_counts.retained_heap_charge(work)?)
    }
}
