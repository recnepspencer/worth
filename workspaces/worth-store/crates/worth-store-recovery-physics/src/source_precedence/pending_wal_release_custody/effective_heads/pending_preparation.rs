//! Reserve and fold the single pending head roster before WAL/root effects.
//! Final publication authority is still checked by `admit_pending` later.

use worth_store_physical_format::ReleaseCustodyHeadMutationV1;

use super::{
    fold_entries, retained_storage, unmoved_checkpoint_head_tree,
    EffectiveReleaseHeadDenial as Denial, PendingReleaseCheckpointBase,
    VerifiedPendingWalReleaseCustody,
};

impl VerifiedPendingWalReleaseCustody {
    /// This is only storage preparation, not an effective selected claim.
    /// The finalized published root and topology are unavailable before effect.
    pub fn prepare_effective_heads(
        &mut self,
        maximum_entries: u64,
        maximum_additional_resident_bytes: u64,
    ) -> Result<u64, Denial> {
        if self.prepared_effective_heads.is_some()
            || self.published_root.is_some()
            || self.verified_transition.is_some()
            || self.ordered_history.is_some()
            || !self.historical_batches.is_empty()
            || !self.ordered_released_batches.is_empty()
        {
            return Err(Denial::Source);
        }
        let replay = self
            .selected_head_replay
            .as_ref()
            .ok_or(Denial::MissingReplay)?;
        let (source_heads, checkpoint_ref, checkpoint_frontier) = match &self.base {
            PendingReleaseCheckpointBase::NoRelease(_) => {
                if self.source_root.release_custody_head_root().is_some()
                    || self.source_root.next_release_custody_head_block() != 1
                {
                    return Err(Denial::Source);
                }
                (&[][..], None, 1)
            }
            PendingReleaseCheckpointBase::ReleasedHeadV2(base) => {
                let (head_root, next_block) = unmoved_checkpoint_head_tree(
                    base.selected_root(),
                    base.checkpoint_source_root(),
                    &self.source_root,
                )?;
                (base.selected_heads(), head_root, next_block)
            }
            _ => return Err(Denial::Source),
        };
        if source_heads.len() as u64 > maximum_entries
            || replay.effect().source_root() != checkpoint_ref
            || replay.effect().source_root() != self.source_root.release_custody_head_root()
            || replay.effect().source_next_block() != checkpoint_frontier
            || replay.effect().source_next_block()
                != self.source_root.next_release_custody_head_block()
            || replay.operation() != self.descriptor.custody().request().idempotency()
            || replay.group() != self.member_group
            || replay.canonical_redo_sha256() != self.member_redo_digest
            || replay.fate() != self.operation_fate
        {
            return Err(Denial::Source);
        }
        let ReleaseCustodyHeadMutationV1::Upsert {
            expected_prior,
            next,
        } = replay.effect().mutation()
        else {
            return Err(Denial::Mutation);
        };
        if next.descriptor_record() != self.descriptor_record
            || next.descriptor_frame_sha256() != self.descriptor_frame_sha256
            || next.manifest_record() != self.manifest_record
            || next.manifest_frame_sha256() != self.manifest_frame_sha256
            || next.reservation_record() != self.reservation_record
            || next.reservation_frame_sha256() != self.reservation_frame_sha256
        {
            return Err(Denial::Mutation);
        }
        let result_count = source_heads
            .len()
            .checked_add(usize::from(expected_prior.is_none()))
            .ok_or(Denial::BoundExceeded)?;
        if result_count as u64 > maximum_entries {
            return Err(Denial::BoundExceeded);
        }
        let (mut source_copy, result) = retained_storage::reserve_rosters(
            source_heads.len(),
            result_count,
            maximum_additional_resident_bytes,
        )?;
        source_copy.extend_from_slice(source_heads);
        let effective = fold_entries(source_heads, expected_prior, next, result_count, result)?;
        let prepared = retained_storage::PreparedEffectiveHeadRosterV14 {
            checkpoint_source_heads: source_copy,
            effective_heads: effective,
            ordered_replays: Vec::new(),
        };
        let actual_bytes = prepared
            .new_roster_backing_bytes()
            .ok_or(Denial::BoundExceeded)?;
        self.prepared_effective_heads = Some(prepared);
        Ok(actual_bytes)
    }
}
