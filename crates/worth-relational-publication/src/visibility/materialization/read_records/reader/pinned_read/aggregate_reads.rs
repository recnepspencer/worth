//! Bind aggregate reads to the immutable Native pin.
use super::RelationalPinnedRead;
use mutation::aggregate_identity::pinned::denial::PinnedAggregateReadDenial;
use mutation::aggregate_identity::pinned::lineage::read_aggregate_lineage;
use mutation::aggregate_identity::pinned::lineage::PinnedAggregateLineageRead;
use mutation::aggregate_identity::pinned::lineage::PinnedAggregateLineageScope;
use mutation::aggregate_identity::pinned::read_aggregate_scope;
use mutation::aggregate_identity::pinned::result::PinnedAggregateRead;
use mutation::aggregate_identity::pinned::PinnedAggregateReadScope;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port::graph_member::key::NativeMemberKey;
use worth_relational_mutation::internal_port as mutation;

impl<Snapshot> RelationalPinnedRead<Snapshot> {
    /// Read one complete stored group and all its incident lineage from this
    /// exact root/schema. Work grows with its relation-kind roster, group and
    /// raw incident adjacency. No current catalog, whole-graph materialization
    /// or identity-only lookup is used.
    pub fn read_aggregate(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: PinnedAggregateReadScope<'_>,
    ) -> Result<PinnedAggregateRead, PinnedAggregateReadDenial> {
        read_aggregate_scope(
            self.pinned_basis(),
            self.candidate_aggregate(),
            execution,
            scope,
        )
    }

    pub fn read_projected_aggregate(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: PinnedAggregateReadScope<'_, NativeMemberKey>,
    ) -> Result<PinnedAggregateRead<NativeMemberKey>, PinnedAggregateReadDenial> {
        read_aggregate_scope(
            self.pinned_basis(),
            self.candidate_aggregate(),
            execution,
            scope,
        )
    }

    /// Scan the selected schema roster and occupied entity slots once, then
    /// follow actual merge targets. Work includes radix/slot navigation and
    /// raw incident adjacency at each hop; no constant-time lookup is claimed.
    pub fn read_aggregate_lineage(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: PinnedAggregateLineageScope<'_>,
    ) -> Result<PinnedAggregateLineageRead, PinnedAggregateReadDenial> {
        read_aggregate_lineage(self.pinned_basis(), execution, scope)
    }
}
