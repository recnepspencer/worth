//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::invariant_projection::verification::ConsumedOutputVerificationStop;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::derived::SettlementRegistrationStop;

impl ConsumedOutputEvidence {
    #[doc(hidden)]
    pub fn establish_restored_before_commit_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source_owner: &WorthQueryRelationalSourceOwner,
    ) -> Result<(), ConsumedOutputVerificationStop> {
        Self::establish_restored_before_commit(self, execution, source_owner)
    }
}

impl ConsumedOutputEvidence {
    #[doc(hidden)]
    pub fn claim_verified_checkpoint_root_for_test(
        &self,
        owner: &SourceInvalidationOwner,
    ) -> Result<bool, SettlementRegistrationStop> {
        Self::claim_verified_checkpoint_root(self, owner)
    }
}
