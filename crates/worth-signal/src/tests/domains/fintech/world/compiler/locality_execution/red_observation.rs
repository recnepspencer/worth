use super::{
    CompiledFinancialLocalityWorld, FinancialPerformedCanonicalWork, LocalitySemanticOutputId,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::tests::domains::fintech) struct FinancialLocalityRedObservation {
    pub(in crate::tests::domains::fintech) performed_counters:
        crate::data::telemetry::SignalInvalidationRealizedCounters,
    pub(in crate::tests::domains::fintech) lineage_records: usize,
    pub(in crate::tests::domains::fintech) direct_candidates_examined: u64,
    pub(in crate::tests::domains::fintech) reverse_candidates_returned: u64,
    pub(in crate::tests::domains::fintech) reverse_bucket_probes: u64,
    pub(in crate::tests::domains::fintech) contract_rejections: u64,
    pub(in crate::tests::domains::fintech) causality_rejections: u64,
    pub(in crate::tests::domains::fintech) nodes_visited: u64,
    pub(in crate::tests::domains::fintech) transitive_frontier_width: u64,
    pub(in crate::tests::domains::fintech) comparator_suppressed_count: u64,
    pub(in crate::tests::domains::fintech) work_items_admitted: u64,
    pub(in crate::tests::domains::fintech) work_items_merged: u64,
    pub(in crate::tests::domains::fintech) ready_items_enqueued: u64,
    pub(in crate::tests::domains::fintech) ready_items_popped: u64,
    pub(in crate::tests::domains::fintech) peak_ready_width: u64,
    pub(in crate::tests::domains::fintech) retained_ready_width: u64,
    pub(in crate::tests::domains::fintech) evaluated_outputs:
        std::collections::BTreeSet<LocalitySemanticOutputId>,
    pub(in crate::tests::domains::fintech) baseline_retained_outputs:
        std::collections::BTreeSet<LocalitySemanticOutputId>,
    pub(in crate::tests::domains::fintech) performed_work: FinancialPerformedCanonicalWork,
    pub(in crate::tests::domains::fintech) execution_stage_outcomes:
        Vec<crate::logic::planner::StageExecutionOutcome>,
    pub(in crate::tests::domains::fintech) physical_ready_batches: u64,
    pub(in crate::tests::domains::fintech) physical_ready_peak: u64,
    pub(in crate::tests::domains::fintech) explanation_fact_count: usize,
    pub(in crate::tests::domains::fintech) provenance_fact_count: usize,
    pub(in crate::tests::domains::fintech) frontier_summary_retained: bool,
    pub(in crate::tests::domains::fintech) replay_event_count: usize,
    pub(in crate::tests::domains::fintech) flow_summary_retained: bool,
}

pub(super) struct RedObservationInput {
    pub(super) before: crate::data::telemetry::InvalidationTelemetry,
    pub(super) after: crate::data::telemetry::InvalidationTelemetry,
    pub(super) evaluation_before: crate::data::telemetry::EvaluationTelemetry,
    pub(super) evaluation_after: crate::data::telemetry::EvaluationTelemetry,
    pub(super) evaluated_outputs: std::collections::BTreeSet<LocalitySemanticOutputId>,
    pub(super) baseline_retained_outputs: std::collections::BTreeSet<LocalitySemanticOutputId>,
    pub(super) performed: crate::data::telemetry::SignalInvalidationRealizedCounters,
    pub(super) execution_stage_outcomes: Vec<crate::logic::planner::StageExecutionOutcome>,
    pub(super) physical_ready: super::PhysicalReadyWitness,
    pub(super) lineage_records: usize,
    pub(super) explanation_fact_count: usize,
    pub(super) provenance_fact_count: usize,
    pub(super) frontier_summary_retained: bool,
    pub(super) replay_event_count: usize,
    pub(super) flow_summary_retained: bool,
}

impl CompiledFinancialLocalityWorld {
    pub(super) fn red_observation(
        &self,
        input: RedObservationInput,
    ) -> FinancialLocalityRedObservation {
        let RedObservationInput {
            before,
            after,
            evaluation_before,
            evaluation_after,
            evaluated_outputs,
            baseline_retained_outputs,
            performed,
            execution_stage_outcomes,
            physical_ready,
            lineage_records,
            explanation_fact_count,
            provenance_fact_count,
            frontier_summary_retained,
            replay_event_count,
            flow_summary_retained,
        } = input;
        FinancialLocalityRedObservation {
            performed_counters: performed,
            lineage_records,
            direct_candidates_examined: delta(
                before.direct_subscriber_candidates_examined,
                after.direct_subscriber_candidates_examined,
            ),
            reverse_candidates_returned: delta(
                before.reverse_subscription_candidates_returned,
                after.reverse_subscription_candidates_returned,
            ),
            reverse_bucket_probes: delta(
                before.reverse_subscription_bucket_probes,
                after.reverse_subscription_bucket_probes,
            ),
            contract_rejections: delta(
                before.direct_contract_rejections,
                after.direct_contract_rejections,
            ),
            causality_rejections: delta(
                before.direct_causality_rejections,
                after.direct_causality_rejections,
            ),
            nodes_visited: delta(
                before.invalidation_nodes_visited,
                after.invalidation_nodes_visited,
            ),
            transitive_frontier_width: delta(
                before.transitive_frontier_width,
                after.transitive_frontier_width,
            ),
            comparator_suppressed_count: delta(
                evaluation_before.skipped_by_comparator,
                evaluation_after.skipped_by_comparator,
            ),
            work_items_admitted: performed.work_items_admitted(),
            work_items_merged: performed.work_items_merged(),
            ready_items_enqueued: performed.ready_items_enqueued(),
            ready_items_popped: performed.ready_items_popped(),
            peak_ready_width: performed.maximum_ready_frontier_width(),
            retained_ready_width: performed.retained_ready_frontier_width(),
            evaluated_outputs,
            baseline_retained_outputs,
            performed_work: self.performed_canonical_work(),
            execution_stage_outcomes,
            physical_ready_batches: physical_ready.nonempty_batches,
            physical_ready_peak: physical_ready.maximum_items,
            explanation_fact_count,
            provenance_fact_count,
            frontier_summary_retained,
            replay_event_count,
            flow_summary_retained,
        }
    }
}

fn delta(before: u64, after: u64) -> u64 {
    after
        .checked_sub(before)
        .expect("locality telemetry is monotonic")
}

pub(super) fn lineage_delta(before: Option<u64>, after: Option<u64>) -> usize {
    match (before, after) {
        (None, None) => 0,
        (None, Some(_)) => 1,
        (Some(before), Some(after)) => after.saturating_sub(before) as usize,
        (Some(_), None) => 0,
    }
}
