//! Explicit finite repair of the disposable terminal correlation index.

use std::num::NonZeroUsize;

use worth_runtime_world::facade::{CompositeComponentChangePosture, RuntimeWorldPublicationRow};

use super::reconstruct::pair_world_history;
use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::provider::{
    WorthQueryCanonicalInboundCompletion, WorthQueryInboundCompletionReadDenial,
    WorthQueryInboundTerminalIndexDenial as Denial,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Spend one bounded World page and at most `maximum_changed_records` on
    /// each Relational commit in it. `Ok(false)` means a resumable cursor is
    /// retained and ordinary lookups remain unavailable until `Ok(true)`.
    pub(in crate::domain_computation) fn rebuild_completed_inbound_index(
        &self,
        maximum_world_slots: NonZeroUsize,
        maximum_changed_records: NonZeroUsize,
        maximum_ancestry_commits: NonZeroUsize,
    ) -> Result<bool, Denial> {
        let (cursor, generation) = self.primary_provider.inbound_reconstruction_cursor();
        let inspection = self.product_runtime.owner.inspection_port();
        let page = inspection
            .performed_publication_page(cursor.as_ref(), maximum_world_slots)
            .map_err(|_| {
                self.primary_provider.abandon_inbound_reconstruction();
                Denial::IndexUnavailable
            })?;
        if page.pending() {
            self.primary_provider.abandon_inbound_reconstruction();
            return Err(Denial::IndexUnavailable);
        }
        let mut entries = Vec::new();
        for world in page.rows() {
            match self.rebuild_one_terminal(
                world,
                maximum_changed_records,
                maximum_ancestry_commits,
            ) {
                Ok(Some(entry)) => entries.push(entry),
                Ok(None) => {}
                Err(denial) => {
                    self.primary_provider.abandon_inbound_reconstruction();
                    return Err(denial);
                }
            }
        }
        if page.next_after().is_none()
            && !inspection
                .publication_frontier_is_current(page.frontier())
                .map_err(|_| Denial::IndexUnavailable)?
        {
            self.primary_provider.abandon_inbound_reconstruction();
            return Err(Denial::IndexUnavailable);
        }
        self.primary_provider.advance_inbound_reconstruction(
            generation,
            page.frontier(),
            page.next_after().cloned(),
            entries,
        )
    }

    fn rebuild_one_terminal(
        &self,
        world: &RuntimeWorldPublicationRow,
        maximum_changed_records: NonZeroUsize,
        maximum_ancestry_commits: NonZeroUsize,
    ) -> Result<Option<WorthQueryCanonicalInboundCompletion>, Denial> {
        let Some(relational) = world.commit().relational_publication_identity() else {
            return Ok(None);
        };
        let row = self
            .primary_provider
            .completion_row_from_commit_patch(relational, maximum_changed_records)
            .map_err(|denial| match denial {
                WorthQueryInboundCompletionReadDenial::ReconstructionWorkExhausted => {
                    Denial::ReconstructionWorkExhausted
                }
                _ => Denial::IndexUnavailable,
            })?;
        let Some(row) = row else {
            return Ok(None);
        };
        if row.original_incarnation_ordinal != world.product_incarnation().ordinal()
            || world.commit().relational_change() != CompositeComponentChangePosture::Published
            || world
                .component_results()
                .relational_commit_result()
                .is_none()
            || world.component_results().relational_settlement() != Some(&row.completion_commit)
        {
            return Err(Denial::WorldPairMismatch);
        }
        let history = self
            .product_runtime
            .owner
            .inspection_port()
            .trace_ancestry(world.commit().identity().clone(), maximum_ancestry_commits)
            .map_err(|_| Denial::IndexUnavailable)?;
        let (original, completion, attempt) =
            pair_world_history(&history, &row, world.product_incarnation())?;
        if &completion != world.commit().identity() || &attempt != world.publication_attempt() {
            return Err(Denial::WorldPairMismatch);
        }
        Ok(Some(
            WorthQueryCanonicalInboundCompletion::from_verified_row(
                row,
                world.product_incarnation(),
                original,
                completion,
                attempt,
            ),
        ))
    }
}
