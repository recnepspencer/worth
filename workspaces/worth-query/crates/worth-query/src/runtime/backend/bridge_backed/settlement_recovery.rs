use super::super::WorthQuerySettlementRecoveryBackend;
use super::WorthQueryBridgeBackedRuntimeBackend;

impl WorthQuerySettlementRecoveryBackend for WorthQueryBridgeBackedRuntimeBackend {
    fn repair_pending_branch_merge_settlement(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: worth_relational::facade::history::CommitId,
    ) -> Result<
        worth_relational::facade::history::RelationalCommitReceipt,
        crate::runtime::WorthQuerySettlementRepairError,
    > {
        self.relational_runtime
            .as_mut()
            .ok_or(crate::runtime::WorthQuerySettlementRepairError::RelationalOwnerUnavailable)?
            .execute_mutation(execution, |runtime| {
                runtime.repair_pending_publication_settlement(commit_id)
            })
            .map_err(crate::runtime::WorthQuerySettlementRepairError::PrimaryGraphIndexRefresh)?
            .map_err(Into::into)
    }
}
