//! Exact descriptive lookup in the existing owner registry, without claim issuance.
use super::*;
use worth_foundational::facade::RecordedProductBranchReference;

impl WorthQueryRecoveryHandleRegistry {
    pub fn recorded_claim_state(
        &self,
        provider_runtime_instance_id: u64,
        original: &RecordedProductBranchReference,
    ) -> Result<(bool, Option<RecoveryClaimDisposition>), worth_query_execution_contracts::facade::QueryDispatchJournalDenial>{
        let claim = match original {
            RecordedProductBranchReference::Home(head) => {
                WorthQueryRecoveryMintClaim::Home(head.selected_commit())
            }
            RecordedProductBranchReference::Memory(head) => WorthQueryRecoveryMintClaim::Memory {
                provider_runtime_instance_id,
                world_owner: head.owner(),
                commit_ordinal: head.selected_commit(),
            },
        };
        self.claim_state(&claim)
    }
}
