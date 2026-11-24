//! Walk execution from an exact pinned read.
use super::super::super::pinned_read::RelationalPinnedRead;
use super::{PinnedStructuralWalk, PinnedStructuralWalkDenial, PinnedStructuralWalkScope};
use worth_execution::ExecutionRequest;

impl<Snapshot> RelationalPinnedRead<Snapshot> {
    /// One global multi-source traversal. Success exhausts the frontier; no
    /// partial result escapes a stop. The returned owner retains its payer.
    pub fn complete_relation_walk(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: PinnedStructuralWalkScope<'_>,
    ) -> Result<PinnedStructuralWalk, PinnedStructuralWalkDenial> {
        super::complete::complete(execution, scope, self.into())
    }
}
