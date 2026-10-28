pub trait WorthQuerySettlementRecoveryBackend {
    fn repair_pending_branch_merge_settlement(
        &mut self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _commit_id: worth_relational::facade::history::CommitId,
    ) -> Result<
        worth_relational::facade::history::RelationalCommitReceipt,
        crate::runtime::WorthQuerySettlementRepairError,
    > {
        Err(crate::runtime::WorthQuerySettlementRepairError::RelationalOwnerUnavailable)
    }
}
