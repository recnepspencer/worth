//! Compare a completed no-effect run at the unchanged current Native input.
use super::*;
impl PreparedSourceFactRebase {
    pub(in crate::domain_computation::primary_graph) fn verify_executed_adoption(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        adoption: &worth_relational::facade::mvcc::RelationalCurrentOutputAdoption,
        correspondence: &crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
        control: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl<'_, '_>,
    ) -> Result<FactPayload, RebaseVerificationFailure> {
        let selected = runtime
            .read_truth()
            .positioned_snapshot(snapshot)
            .map_err(|_| RebaseVerificationFailure::native_unavailable())?;
        if selected.runtime_instance_id() != adoption.basis().runtime_instance_id()
            || selected.root_id() != adoption.basis().root_identity()
            || selected.branch_id() != adoption.basis().branch_id()
        {
            return Err(RebaseVerificationFailure::native_unavailable());
        }
        let rebased = rebase_output(
            execution,
            runtime,
            snapshot,
            self,
            correspondence,
            &[],
            None,
            true,
            control,
        );
        if let Some(ordinal) = rebased.superseded_by_own_effect().next() {
            return Err(RebaseVerificationFailure::native_fact_unavailable(ordinal));
        }
        rebased.retained().map_err(|failed| failed.reason)
    }
}
