use super::super::{
    self as history, RelationalAuthoredManagedSourceSelector,
    RelationalAuthoredWriteDenial as Denial,
};
use crate::identity::data::EntityId;
use crate::runtime::RelationalRuntime;
use crate::validation::candidate_source::program::ManagedIndexSourceProgram;
use worth_execution::ExecutionRequest;

impl RelationalRuntime {
    /// Issue a declared Source impact-candidate coordinate. Shared-row scopes
    /// and disjoint fields on declared field kinds can be conservative candidates.
    /// No principal, row,
    /// Query proof, mutation eligibility or Source READ is admitted here.
    pub fn authored_managed_source_selector(
        &self,
        execution: ExecutionRequest<'_, '_>,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
    ) -> Result<RelationalAuthoredManagedSourceSelector, Denial> {
        history::run(execution, |context| {
            let catalog = &self
                .schema_contract_runtime
                .custom_invariant_registries
                .source_history;
            history::admit_work(context, catalog.len() as u64)?;
            catalog.selector(program, scope)
        })
    }
}
