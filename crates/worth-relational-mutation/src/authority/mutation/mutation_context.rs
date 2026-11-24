use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistry;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

pub(crate) struct MutationContext<'a> {
    pub(crate) execution: worth_execution::ExecutionRequest<'a, 'a>,
    pub(crate) deferred_publication_edges:
        Option<&'a std::collections::BTreeSet<worth_relational_contracts::internal_port::identity::data::record_ids::RelationId>>,
    pub(crate) state: &'a mut WorkingState,
    pub(crate) schema: &'a RelationalSchemaRegistry,
    pub(crate) record_allocations:
        &'a mut dyn worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::allocation_batch::MutationRecordAllocations,
}
