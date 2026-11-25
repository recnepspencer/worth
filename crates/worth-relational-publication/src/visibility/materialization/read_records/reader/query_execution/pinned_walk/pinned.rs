//! Walk execution from an exact pinned read.
use worth_relational_branch::internal_port as branch;
use super::super::super::pinned_read::RelationalPinnedRead;
use branch::visibility::materialization::read_records::reader::query_execution::pinned_walk::outcome::PinnedStructuralWalk;
use branch::visibility::materialization::read_records::reader::query_execution::pinned_walk::outcome::PinnedStructuralWalkDenial;
use branch::visibility::materialization::read_records::reader::query_execution::pinned_walk::PinnedStructuralWalkScope;
use worth_execution::ExecutionRequest;

impl<Snapshot> RelationalPinnedRead<Snapshot> {
    /// One global multi-source traversal. Success exhausts the frontier; no
    /// partial result escapes a stop. The returned owner retains its payer.
    pub fn complete_relation_walk(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: PinnedStructuralWalkScope<'_>,
    ) -> Result<PinnedStructuralWalk, PinnedStructuralWalkDenial> {
        branch::visibility::materialization::read_records::reader::query_execution::pinned_walk::complete::complete(execution, scope, self.into())
    }
}
