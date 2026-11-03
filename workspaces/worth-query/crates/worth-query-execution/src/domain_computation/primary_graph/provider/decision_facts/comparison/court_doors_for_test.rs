//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn capture_application_read_set_for_test(
        &self, _execution: &worth_execution::ExecutionRequest<'_, '_>, authority: &WorthQuerySessionReadAuthority<'_>, control: StorageControl<'_, '_>) -> Result<WorthQueryCompleteDecisionReadSetReceipt, WorthQueryDecisionReadSetFailure> {
        Self::capture_application_read_set(self, _execution, authority, control)
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn compare_application_read_set_for_test(
        &self, execution: &worth_execution::ExecutionRequest<'_, '_>, authority: &WorthQuerySessionReadAuthority<'_>, receipt: WorthQueryCompleteDecisionReadSetReceipt, control: StorageControl<'_, '_>) -> Result<WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryDecisionReadSetFailure> {
        Self::compare_application_read_set(self, execution, authority, receipt, control)
    }
}
