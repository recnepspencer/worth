//! Select one stored orbit or component through the aggregate read owner.
use super::RelationalPinnedRead;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::AggregateDefinitionIdentity;
use worth_relational_contracts::internal_port::{
    graph_member::key::NativeMemberKey,
    identity::data::{primitives::KindId, record_ids::EntityId},
};
use worth_relational_mutation::internal_port::aggregate_identity::pinned::{
    PinnedAggregateRead, PinnedAggregateReadDenial, PinnedAggregateReadScope,
};

impl<Snapshot> RelationalPinnedRead<Snapshot> {
    /// Read the complete orbit for this member and installed relation kind.
    pub fn orbit_of(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        relation: KindId,
        member: EntityId,
    ) -> Result<PinnedAggregateRead, PinnedAggregateReadDenial> {
        self.read_aggregate(execution, PinnedAggregateReadScope::new(relation, member))
    }

    /// Read a complete group selected by its installed component definition.
    pub fn component_of(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        definition: AggregateDefinitionIdentity,
        member: impl Into<NativeMemberKey>,
    ) -> Result<PinnedAggregateRead<NativeMemberKey>, PinnedAggregateReadDenial> {
        self.read_projected_aggregate(
            execution,
            PinnedAggregateReadScope::for_member_definition(definition, member.into()),
        )
    }
}
