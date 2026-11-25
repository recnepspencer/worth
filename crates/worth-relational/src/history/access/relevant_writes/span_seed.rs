//! Span-bounded ordinary seed of the authored-write accelerator.
use super::{RelationalAuthoredWriteDenial as Denial, RelationalAuthoredWriteSpan};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::change_source::RelationalSelectedCommit;
use crate::runtime::RelationalRuntime;
use std::num::NonZeroUsize;
use worth_execution::ExecutionRequest;

impl RelationalRuntime {
    /// Make the target's generation cover `original` when it does not.
    ///
    /// A reopened runtime holds no generation, so a history cursor for an
    /// older original refuses `GenerationUnavailable` until one exists. This
    /// seed walks parents from `target` back to `original` only, and extends a
    /// generation over that span, with the same per-commit work the commit path
    /// does. Cost: O(span × log span + authored writes in span × log keys),
    /// charged to this request and independent of the history before
    /// `original`. If the target's generation already holds the original, it
    /// returns `Covered` after one lookup.
    ///
    /// Refusals: `NotSelectedPrefix` when the root is reached first,
    /// `UnsupportedMergeInterval` at a merge, `SpanBoundReached` after
    /// `maximum_links` parent links. The install is widen-only and never
    /// persisted, so the generation is always correct or absent. This is not
    /// the cold reconstruction lane (`rebuild_authored_write_generation`).
    pub fn seed_authored_write_span(
        &self,
        execution: ExecutionRequest<'_, '_>,
        original: &AdmittedRelationalBranchBasis,
        target: RelationalSelectedCommit,
        maximum_links: Option<NonZeroUsize>,
    ) -> Result<RelationalAuthoredWriteSpan, Denial> {
        super::maintenance::require_exact(self, &target)?;
        let original = original
            .observation()
            .commit_id()
            .ok_or(Denial::NoSelectedCommit)?;
        self.history.seed_authored_write_span(
            execution,
            original,
            target.commit_id(),
            maximum_links,
        )
    }
}
