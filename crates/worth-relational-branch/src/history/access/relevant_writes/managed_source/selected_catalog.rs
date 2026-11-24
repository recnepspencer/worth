use super::super::{
    self as history, RelationalAuthoredManagedSourceSelector,
    RelationalAuthoredWriteDenial as Denial,
};
use crate::identity::data::EntityId;
use crate::validation::candidate_source::program::ManagedIndexSourceProgram;
use worth_execution::ExecutionRequest;

pub struct ManagedSourceHistoryReads<'a> {
    pub runtime_instance_id: u64,
    pub catalog: &'a std::sync::Arc<
        crate::validation::candidate_source::authored_history::catalog::SourceHistoryCatalog,
    >,
}

impl ManagedSourceHistoryReads<'_> {
    pub fn select(
        &self,
        execution: ExecutionRequest<'_, '_>,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
    ) -> Result<RelationalAuthoredManagedSourceSelector, Denial> {
        history::run(execution, |context| {
            history::admit_work(context, self.catalog.len() as u64)?;
            self.catalog.selector(program, scope)
        })
    }
}
