use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::overlay::WorkingState;

pub(crate) struct MutationContext<'a> {
    pub(crate) execution: worth_execution::ExecutionRequest<'a, 'a>,
    pub(crate) state: &'a mut WorkingState,
    pub(crate) schema: &'a RelationalSchemaRegistry,
    pub(crate) record_allocations: &'a mut dyn crate::runtime::MutationRecordAllocations,
}
