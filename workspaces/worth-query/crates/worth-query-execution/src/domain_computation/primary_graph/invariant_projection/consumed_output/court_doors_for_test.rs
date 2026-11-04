//! Sealed test doors over this module's private members.
use super::*;

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
