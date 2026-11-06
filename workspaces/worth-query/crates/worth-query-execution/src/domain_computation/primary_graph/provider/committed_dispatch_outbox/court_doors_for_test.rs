//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn observe_expected_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &super::super::WorthQueryCommittedDispatchOutboxBinding,
        expected_commit: &worth_relational::facade::history::RelationalCommitReceipt,
        expected_runtime: u64,
    ) -> Result<WorthQueryCommittedDispatchOutboxOwnerObservation, Denial> {
        Self::observe_expected(self, execution, binding, expected_commit, expected_runtime)
    }
}
