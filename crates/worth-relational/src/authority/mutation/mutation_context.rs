use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::overlay::WorkingState;

pub(crate) struct MutationContext<'a> {
    pub(crate) state: &'a mut WorkingState,
    pub(crate) symbols: &'a crate::symbols::data::StringInterner,
    pub(crate) schema: &'a RelationalSchemaRegistry,
    pub(crate) record_allocations: &'a mut crate::runtime::PendingRecordAllocations,
}
