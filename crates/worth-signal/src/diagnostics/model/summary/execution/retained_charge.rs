use super::ExecutionReportSummary;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for ExecutionReportSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            profile: _,
            stage_count: _,
            task_count: _,
            tasks_executed: _,
            tasks_pruned: _,
            tasks_validated_clean: _,
            tasks_deferred_by_condition: _,
            tasks_reverted_clean_by_condition: _,
            tasks_satisfied_by_memoization: _,
            tasks_with_suppressed_propagation: _,
            prepared_evaluations_produced: _,
            prepared_evaluations_applied: _,
            dependency_capture_updates: _,
            semantic_segment_count: _,
            temporal_summary,
            task_outcome_counts,
            stage_outcome_counts,
        } = self;
        temporal_summary
            .retained_heap_charge(work)?
            .checked_add(task_outcome_counts.retained_heap_charge(work)?)?
            .checked_add(stage_outcome_counts.retained_heap_charge(work)?)
    }
}
