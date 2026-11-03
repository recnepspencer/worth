//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    #[doc(hidden)]
    pub fn require_current_read_for_test(
        &self, execution: &worth_execution::ExecutionRequest<'_, '_>, relational: &worth_relational::facade::runtime::RelationalRuntime, read: &RetainedOutputCurrentnessRead, receipt: Option<&WorthQueryApplicationCommitReceipt>) -> Result<(), WorthQueryOutputDemandDenial> {
        Self::require_current_read(self, execution, relational, read, receipt)
    }
}
