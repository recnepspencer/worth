//! Complete ordered history attachment for one or more completed releases under
//! the selected checkpoint, followed by this claim's pending final release.

use std::sync::Arc;

use super::{
    PendingReleaseCheckpointBase, PendingWalReleaseCustodyDenial as Denial,
    VerifiedOrderedPendingWalReleaseBatch, VerifiedPendingWalReleaseCustody,
};
use crate::{VerifiedOrderedRootEdge, VerifiedOrderedRootHistory};

impl VerifiedPendingWalReleaseCustody {
    pub fn attach_ordered_history(
        &mut self,
        history: Arc<VerifiedOrderedRootHistory>,
        batches: Vec<VerifiedOrderedPendingWalReleaseBatch>,
        maximum_retained_bytes: u64,
    ) -> Result<(), Denial> {
        let supported_base = match &self.base {
            PendingReleaseCheckpointBase::NoRelease(_) => true,
            PendingReleaseCheckpointBase::ReleasedHeadV2(base) => {
                base.source_root_sha256() == history.checkpoint_root_frame_sha256()
                    && base.selected_root() == &self.source_root
                    && base.selected_root_sha256() == history.selected_root_frame_sha256()
            }
            PendingReleaseCheckpointBase::Released(_)
            | PendingReleaseCheckpointBase::ReleasedAddressed(_) => false,
        };
        if !supported_base
            || self.ordered_history.is_some()
            || self.prepared_effective_heads.is_some()
            || !self.historical_batches.is_empty()
            || batches.is_empty()
            || self.checkpoint_source_root_sha256 != history.checkpoint_root_frame_sha256()
            || self.source_root_sha256 != history.selected_root_frame_sha256()
            || self.descriptor.custody().source_free_space_frame_sha256()
                != history.selected_free_space_frame_sha256()
        {
            return Err(Denial::SourceBinding);
        }
        let mut previous_lsn_end = 0;
        let mut release_ordinal = 0usize;
        let mut retained_bytes = history.peak_scratch_bytes();
        for (index, edge) in history.edges().iter().enumerate() {
            let VerifiedOrderedRootEdge::Released(released) = edge else {
                continue;
            };
            let Some(batch) = batches.get(release_ordinal) else {
                return Err(Denial::ControlBinding);
            };
            if batch.edge_index() != index
                || batch.descriptor_frame().record() != released.descriptor_record()
                || batch.descriptor_frame().payload_sha256() != released.descriptor_frame_sha256()
                || batch.descriptor().custody().request().idempotency() != released.operation()
                || batch.raw_fate() != released.fate()
                || batch.wal_fate().lsn_start() != released.lsn().start().get()
                || batch.wal_fate().lsn_end_exclusive() != released.lsn().end_exclusive().get()
                || released.lsn().start().get() < previous_lsn_end
            {
                return Err(Denial::ControlBinding);
            }
            retained_bytes = retained_bytes
                .checked_add(batch.retained_bytes())
                .ok_or(Denial::ControlBinding)?;
            previous_lsn_end = released.lsn().end_exclusive().get();
            release_ordinal += 1;
        }
        let final_lsn_end = match history.edges().last() {
            Some(VerifiedOrderedRootEdge::Ordinary(step)) => step
                .lsn_range()
                .ok_or(Denial::DurableWalFate)?
                .end_exclusive()
                .get(),
            Some(VerifiedOrderedRootEdge::Released(step)) => step.lsn().end_exclusive().get(),
            // A retirement edge is only ever first, so a history ending in one
            // has no released edge for this pending release to follow.
            Some(VerifiedOrderedRootEdge::Retirement(_)) | None => {
                return Err(Denial::SourceBinding)
            }
        };
        if release_ordinal != batches.len()
            || final_lsn_end > self.wal_fate.lsn_start()
            || retained_bytes > maximum_retained_bytes
            || history.edges().iter().any(|edge| {
                edge.operation() == Some(self.descriptor.custody().request().idempotency())
            })
        {
            return Err(Denial::ControlBinding);
        }
        self.ordered_history = Some(history);
        self.ordered_released_batches = batches.into_boxed_slice();
        Ok(())
    }
}
