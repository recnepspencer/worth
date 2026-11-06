//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationObservedFact {
    #[doc(hidden)]
    pub fn remains_equal_in_for_test(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<bool, crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial>{
        Self::remains_equal_in(self, runtime, execution, snapshot)
    }
}
