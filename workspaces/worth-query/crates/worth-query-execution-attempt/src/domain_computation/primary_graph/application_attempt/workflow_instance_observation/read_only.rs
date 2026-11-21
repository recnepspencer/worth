//! Exact state/history reconstruction without progression cache, allowance or deadlines.
use super::*;
/// Native retained history data only; contains no permission or actor identities.
pub struct ReadOnlyWorkflowHistory {
    history: history::ObservedWorkflowHistory,
}
impl ReadOnlyWorkflowHistory {
    pub fn matches_target(
        &self,
        instance: EntityId,
        basis: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> bool {
        matches!(self.history.facts.as_slice(),[WorthQueryApplicationObservedFact::WorkflowHistoryBasis { instance:recorded,snapshot,.. }]
            if *recorded==instance && snapshot.runtime_instance_id()==basis.runtime_instance_id()
                && snapshot.branch_id()==basis.branch_id() && snapshot.snapshot_id()==basis.snapshot_id()
                && snapshot.version_id()==basis.version_id())
    }

    pub fn transitions(&self) -> &[ObservedWorkflowTransition] {
        &self.history.transitions
    }

    pub fn transition_count(&self) -> usize {
        self.history.transitions.len()
    }
    pub fn recorded_paths(&self) -> &[String] {
        &self.history.recorded_paths
    }
}
/// Only the governed reader calls this after independent subject admission.
/// No recorded actor fields are read; audit has its separate admission and owner.
pub fn observe_read_only_history(
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
