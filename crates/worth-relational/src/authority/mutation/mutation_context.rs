use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::overlay::WorkingState;

pub(crate) struct MutationContext<'a> {
    pub(crate) deferred_publication_edges:
        Option<&'a std::collections::BTreeSet<crate::identity::data::RelationId>>,
    pub(crate) state: &'a mut WorkingState,
    pub(crate) schema: &'a RelationalSchemaRegistry,
    pub(crate) record_allocations: &'a mut crate::runtime::PendingRecordAllocations,
}
