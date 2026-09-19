use super::{
    ApplySummary, ChangeInputSummary, FlowCauseSample, FlowSummary, InvalidationSummary,
    PlanningSummary, PrecomputeSummary, RollbackSummary,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for ChangeInputSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            changed_nodes,
            changed_aspects,
            changed_region_count: _,
            causality_kind,
        } = self;
        changed_nodes
            .retained_heap_charge(work)?
            .checked_add(changed_aspects.retained_heap_charge(work)?)?
            .checked_add(causality_kind.retained_heap_charge(work)?)
    }
}
impl RetainedStorageMeasurement for InvalidationSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            invalidated_direct_subscribers: _,
            maybe_stale_direct_subscribers: _,
            partition_scoped_checks: _,
            narrowed_frontier_width: _,
            transitive_frontier_width: _,
            frontier_seed_count: _,
            frontier_group_count: _,
            frontier_direct_wave_count: _,
            frontier_transitive_wave_count: _,
            frontier_partition_match_count: _,
            frontier_detail_match_count: _,
            frontier_cycle_check_candidate_count: _,
            frontier_cycle_check_visited_count: _,
            frontier_trace_retained_count: _,
        } = self;
        Ok(Charge::ZERO)
    }
}
impl RetainedStorageMeasurement for FlowSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            profile: _,
            change,
            invalidation,
            planning,
            precompute,
            apply,
            cause_samples,
            event_epochs,
            observation,
            rollback,
            explanation,
        } = self;
        change
            .retained_heap_charge(work)?
            .checked_add(invalidation.retained_heap_charge(work)?)?
            .checked_add(planning.retained_heap_charge(work)?)?
            .checked_add(precompute.retained_heap_charge(work)?)?
            .checked_add(apply.retained_heap_charge(work)?)?
            .checked_add(cause_samples.retained_heap_charge(work)?)?
            .checked_add(event_epochs.retained_heap_charge(work)?)?
            .checked_add(observation.retained_heap_charge(work)?)?
            .checked_add(rollback.retained_heap_charge(work)?)?
            .checked_add(explanation.retained_heap_charge(work)?)
    }
}
impl RetainedStorageMeasurement for FlowCauseSample {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            node: _,
            cause_kinds,
            scope_kinds,
            scope_notes,
            suspect_classes,
            rewired: _,
            conservative_recompute: _,
        } = self;
        cause_kinds
            .retained_heap_charge(work)?
            .checked_add(scope_kinds.retained_heap_charge(work)?)?
            .checked_add(scope_notes.retained_heap_charge(work)?)?
            .checked_add(suspect_classes.retained_heap_charge(work)?)
    }
}
impl RetainedStorageMeasurement for PlanningSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { plan } = self;
        plan.retained_heap_charge(work)
    }
}
impl RetainedStorageMeasurement for PrecomputeSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            executor: _,
            stage_count: _,
            task_count: _,
            prepared_evaluations_produced: _,
            tasks_deferred_by_condition: _,
            tasks_satisfied_by_memoization: _,
        } = self;
        Ok(Charge::ZERO)
    }
}
impl RetainedStorageMeasurement for ApplySummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            report,
            prepared_evaluations_applied: _,
            dependency_capture_updates: _,
            tasks_validated_clean: _,
            tasks_pruned: _,
            tasks_with_suppressed_propagation: _,
        } = self;
        report.retained_heap_charge(work)
    }
}
impl RetainedStorageMeasurement for RollbackSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            rolled_back: _,
            staged_node_patch_count: _,
            max_touched_nodes_in_txn: _,
            reason,
        } = self;
        reason.retained_heap_charge(work)
    }
}
