use super::backend::StatefulBridgeRuntimeBackend;

impl crate::runtime::WorthQuerySettlementRecoveryBackend for StatefulBridgeRuntimeBackend {
    fn repair_deferred_branch_merge_settlement(
        &mut self,
        deferred: &crate::ordinary::workflow::WorthQueryBranchMergeSettlementDeferred,
    ) -> Result<
        worth_relational::facade::history::RelationalCommitReceipt,
        crate::runtime::WorthQuerySettlementRepairError,
    > {
        self.state
            .borrow()
            .relational_source
            .with_runtime_mut(|runtime| {
                runtime.repair_deferred_publication_settlement(deferred.settlement())
            })
            .map_err(Into::into)
    }

    fn repair_pending_branch_merge_settlement(
        &mut self,
        commit_id: worth_relational::facade::history::CommitId,
    ) -> Result<
        worth_relational::facade::history::RelationalCommitReceipt,
        crate::runtime::WorthQuerySettlementRepairError,
    > {
        self.state
            .borrow()
            .relational_source
            .with_runtime_mut(|runtime| runtime.repair_pending_publication_settlement(commit_id))
            .map_err(Into::into)
    }
}
