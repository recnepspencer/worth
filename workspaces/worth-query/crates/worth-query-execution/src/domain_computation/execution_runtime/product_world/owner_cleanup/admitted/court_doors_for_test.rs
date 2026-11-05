//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProductBranchOwnerCleanup {
    #[doc(hidden)]
    pub fn retry_in_request_with_retirements_for_test(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReceipt,
        WorthQueryProductBranchOwnerCleanupFailure,
    > {
        Self::retry_in_request_with_retirements(self, execution, retirements)
    }
}
