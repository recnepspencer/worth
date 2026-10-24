use super::backend::StatefulBridgeRuntimeBackend;

impl crate::runtime::WorthQuerySettlementRecoveryBackend for StatefulBridgeRuntimeBackend {
    fn repair_pending_branch_merge_settlement(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: worth_relational::facade::history::CommitId,
    ) -> Result<
        worth_relational::facade::history::RelationalCommitReceipt,
        crate::runtime::WorthQuerySettlementRepairError,
    > {
        self.state
            .borrow()
            .relational_source
            .with_runtime_mut(request_execution, |runtime| {
                runtime.repair_pending_publication_settlement(commit_id)
            })
            .map_err(Into::into)
    }
}
