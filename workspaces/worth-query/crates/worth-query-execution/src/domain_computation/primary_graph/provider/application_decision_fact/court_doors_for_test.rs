//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphApplicationDecisionFact {
    #[doc(hidden)]
    pub fn locator_identity_for_test(&self) -> String {
        Self::locator_identity(self)
    }
}

impl WorthQueryPrimaryGraphApplicationDecisionFact {
    #[doc(hidden)]
    pub fn remains_equal_in_for_test(
        &self, execution: &worth_execution::ExecutionRequest<'_, '_>, runtime: &worth_relational::facade::runtime::RelationalRuntime, snapshot: &worth_relational::facade::snapshots::SnapshotHandle) -> Result<bool, super::super::super::WorthQueryApplicationAttemptDenial> {
        Self::remains_equal_in(self, execution, runtime, snapshot)
    }
}
