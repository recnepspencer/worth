//! Bind aggregate reads to the immutable Native pin.
use super::RelationalPinnedRead;
use crate::aggregate_identity::{
    read_aggregate_lineage, read_aggregate_scope, PinnedAggregateLineageRead,
    PinnedAggregateLineageScope, PinnedAggregateRead, PinnedAggregateReadDenial,
    PinnedAggregateReadScope,
};
use crate::graph_member::NativeMemberKey;
use worth_execution::ExecutionRequest;

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
