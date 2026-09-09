use super::super::WorthQuerySettlementRecoveryBackend;
use super::WorthQueryBridgeBackedRuntimeBackend;

impl WorthQuerySettlementRecoveryBackend for WorthQueryBridgeBackedRuntimeBackend {
    fn repair_deferred_branch_merge_settlement(
        &mut self,
        deferred: &crate::ordinary::workflow::WorthQueryBranchMergeSettlementDeferred,
    ) -> Result<
        worth_relational::facade::history::RelationalCommitReceipt,
        crate::runtime::WorthQuerySettlementRepairError,
    > {
        let settlement = deferred.settlement();
        self.relational_runtime
            .as_mut()
            .ok_or(crate::runtime::WorthQuerySettlementRepairError::RelationalOwnerUnavailable)?
            .execute_mutation(|runtime| runtime.repair_deferred_publication_settlement(settlement))
            .map_err(crate::runtime::WorthQuerySettlementRepairError::PrimaryGraphIndexRefresh)?
            .map_err(Into::into)
    }

    fn repair_pending_branch_merge_settlement(
        &mut self,
        commit_id: worth_relational::facade::history::CommitId,
    ) -> Result<
        worth_relational::facade::history::RelationalCommitReceipt,
        crate::runtime::WorthQuerySettlementRepairError,
    > {
        self.relational_runtime
            .as_mut()
            .ok_or(crate::runtime::WorthQuerySettlementRepairError::RelationalOwnerUnavailable)?
            .execute_mutation(|runtime| runtime.repair_pending_publication_settlement(commit_id))
            .map_err(crate::runtime::WorthQuerySettlementRepairError::PrimaryGraphIndexRefresh)?
            .map_err(Into::into)
    }
}
