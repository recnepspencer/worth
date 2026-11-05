//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrincipalCurrentnessDependency {
    #[doc(hidden)]
    pub fn remains_current_in_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<
        bool,
        crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind,
    > {
        Self::remains_current_in(self, execution, runtime, snapshot)
    }
}
