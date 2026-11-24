//! Bind Source history to the installed owner and its exact compiled catalog.
use super::RelationalRuntime;
use crate::history::access::relevant_writes::{
    managed_source::selected_catalog::ManagedSourceHistoryReads,
    RelationalAuthoredManagedSourceSelector, RelationalAuthoredWriteDenial,
};
use crate::identity::data::EntityId;
use crate::validation::candidate_source::program::ManagedIndexSourceProgram;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

impl RelationalRuntime {
    /// Issue a declared Source impact-candidate coordinate. Shared-row scopes
    /// and disjoint fields on declared field kinds can be conservative candidates.
    /// No principal, row, Query proof, mutation eligibility or Source READ is admitted.
    pub fn authored_managed_source_selector(
        &self,
        execution: ExecutionRequest<'_, '_>,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
    ) -> Result<RelationalAuthoredManagedSourceSelector, RelationalAuthoredWriteDenial> {
        ManagedSourceHistoryReads::from(self).select(execution, program, scope)
    }
}

impl<'a> From<&'a RelationalRuntime> for ManagedSourceHistoryReads<'a> {
    fn from(runtime: &'a RelationalRuntime) -> Self {
        Self {
            runtime_instance_id: runtime.runtime_instance_id(),
            catalog: &runtime
                .schema_contract_runtime
                .custom_invariant_registries
                .source_history,
        }
    }
}

impl<'a> From<&'a Arc<RelationalRuntime>> for ManagedSourceHistoryReads<'a> {
    fn from(runtime: &'a Arc<RelationalRuntime>) -> Self {
        runtime.as_ref().into()
    }
}
