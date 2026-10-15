//! Exact state/history reconstruction without progression cache, allowance or deadlines.
use super::*;
/// Native retained history data only; contains no permission or actor identities.
pub(in crate::domain_computation::primary_graph) struct ReadOnlyWorkflowHistory {
    history: history::ObservedWorkflowHistory,
}
impl ReadOnlyWorkflowHistory {
    pub(in crate::domain_computation::primary_graph) fn transition_count(&self) -> usize {
        self.history.transitions.len()
    }
    pub(in crate::domain_computation::primary_graph) fn recorded_paths(&self) -> &[String] {
        &self.history.recorded_paths
    }
}
/// Only the governed reader calls this after independent subject admission.
/// No recorded actor fields are read; audit has its separate admission and owner.
pub(in crate::domain_computation::primary_graph::application_attempt) fn observe_read_only_history(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    instance: EntityId,
    maximum_transitions: usize,
    budget: worth_query_installation::facade::WorthQueryWorkflowHistoryReconstructionBudget,
) -> Result<ReadOnlyWorkflowHistory, WorthQueryApplicationAttemptDenial> {
    history::observe(
        execution,
        runtime,
        snapshot,
        layout,
        instance,
        history::HistoryBasis::ReadOnly,
        maximum_transitions,
        history::HistoryNodes::Recorded,
        budget,
    )
    .map(|history| ReadOnlyWorkflowHistory { history })
}
