use crate::history::data::BranchId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CancellationResidueSnapshot {
    branch_reference: crate::branch::RelationalBranchReferenceState,
    immutable_commit_count: usize,
    pending_publication_routes: usize,
    patch_stream: crate::publication::patch::data::PatchStreamBatch,
    retention: crate::storage::data::RetentionPlan,
}

impl CancellationResidueSnapshot {
    pub(super) fn capture(
        runtime: &crate::runtime::RelationalRuntime,
        branch_id: &BranchId,
    ) -> Self {
        let request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&request);
        Self {
            branch_reference: runtime
                .branch_reference_state(branch_id)
                .expect("cancellation evidence branch remains registered"),
            immutable_commit_count: runtime.history().immutable_commit_count(),
            pending_publication_routes: runtime.history.pending_canonical_publication_route_count(),
            patch_stream: runtime
                .publication()
                .read_patch_stream(crate::publication::patch::data::PatchStreamRequest::default())
                .expect("cancellation evidence reads the canonical patch stream"),
            retention: runtime.retention().inspect_plan(execution).unwrap(),
        }
    }
}
