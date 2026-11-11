//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn create_product_branch_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &WorthQueryProductBranchLease,
        source_program: Option<&ApplicationProgramRevision>,
        intent: ProductBranchCreationIntent,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> Result<WorthQueryProductBranchCreationOutcome, WorthQueryProductBranchCreationDenial> {
        Self::create_product_branch(
            self,
            execution,
            source,
            source_program,
            intent,
            cancellation,
            None,
        )
    }
}
