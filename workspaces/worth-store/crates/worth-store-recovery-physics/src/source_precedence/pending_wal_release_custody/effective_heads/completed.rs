//! Fold all completed V14 effects through the exact selected result. An
//! ordinary tail is optional; no pending descriptor is manufactured.

use worth_store_physical_format::{
    ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadMutationV1, ReleaseCustodyHeadRosterDigestV1,
};

use super::{
    apply_entry, EffectiveReleaseHeadDenial as Denial, VerifiedEffectiveReleaseHeadRosterV14,
};
use crate::{
    VerifiedOrderedHistoricalReleaseCustody, VerifiedOrderedReleasedHeadReplayV14,
    VerifiedOrderedRootEdge,
};

impl VerifiedEffectiveReleaseHeadRosterV14 {
    /// Complete checkpoint-to-selected fold. Every release is an
    /// independently witnessed C.9 edge; intervening ordinary edges preserve
    /// the head reference under their own admitted transition.
    pub fn admit_ordered_completed(
        claim: &VerifiedOrderedHistoricalReleaseCustody,
        attachments: Vec<(usize, VerifiedOrderedReleasedHeadReplayV14)>,
        maximum_entries: u64,
        maximum_retained_bytes: u64,
        maximum_additional_resident_bytes: u64,
    ) -> Result<Self, Denial> {
        let history = claim.history();
        let selected = claim.selected_root();
        let (source_heads, checkpoint_ref, checkpoint_frontier) =
            if let Some(base) = claim.selected_head_v2() {
                (
                    base.selected_heads(),
                    base.checkpoint_source_root().release_custody_head_root(),
                    base.checkpoint_source_root()
                        .next_release_custody_head_block(),
                )
            } else if claim.marker().is_some() {
                (&[][..], None, 1)
            } else {
                return Err(Denial::Source);
            };
        if attachments.is_empty()
            || attachments.len() != claim.released_batches().len()
            || history.selected_root_frame_sha256() != claim.selected_root_frame_sha256()
            || maximum_entries == 0
        {
            return Err(Denial::Source);
        }
        let mut release_count = 0usize;
        let mut inserts = 0usize;
        let mut effect_bytes = 0u64;
        for (index, edge) in history.edges().iter().enumerate() {
            let VerifiedOrderedRootEdge::Released(released) = edge else {
                continue;
            };
            let (attached_index, attachment) =
                attachments.get(release_count).ok_or(Denial::Source)?;
            let batch = claim
                .released_batches()
                .get(release_count)
                .ok_or(Denial::Source)?;
            let replay = attachment.replay();
            let ReleaseCustodyHeadMutationV1::Upsert {
                expected_prior,
                next,
            } = replay.effect().mutation()
            else {
                return Err(Denial::Mutation);
            };
            if *attached_index != index
                || batch.edge_index() != index
                || attachment.source_root_frame_sha256() != released.source_root_frame_sha256()
                || attachment.result_root_frame_sha256() != released.result_root_frame_sha256()
                || replay.lsn_range() != Some(released.lsn())
                || replay.operation() != released.operation()
                || replay.group() != released.group()
                || replay.fate() != released.fate()
                || replay.canonical_redo_sha256() != released.redo_sha256()
                || next.descriptor_record() != batch.descriptor_frame().record()
                || next.descriptor_frame_sha256() != batch.descriptor_frame().payload_sha256()
                || next.manifest_record() != batch.manifest_frame().record()
                || next.manifest_frame_sha256() != batch.manifest_frame().payload_sha256()
                || next.reservation_record() != batch.reservation_frame().record()
                || next.reservation_frame_sha256() != batch.reservation_frame().payload_sha256()
            {
                return Err(Denial::Mutation);
            }
            inserts = inserts
                .checked_add(usize::from(expected_prior.is_none()))
                .ok_or(Denial::BoundExceeded)?;
            effect_bytes = effect_bytes
                .checked_add(
                    replay
                        .effect()
                        .framed_bytes()
                        .ok_or(Denial::BoundExceeded)?,
                )
                .and_then(|bytes| {
                    bytes.checked_add(
                        std::mem::size_of::<VerifiedOrderedReleasedHeadReplayV14>() as u64
                    )
                })
                .ok_or(Denial::BoundExceeded)?;
            release_count += 1;
        }
        let final_max = source_heads
            .len()
            .checked_add(inserts)
            .ok_or(Denial::BoundExceeded)?;
        let retained_bytes = (source_heads.len() as u64)
            .checked_add(final_max as u64)
            .and_then(|entries| {
                entries.checked_mul(std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64)
            })
            .and_then(|bytes| bytes.checked_add(effect_bytes))
            .and_then(|bytes| {
                bytes.checked_add((attachments.capacity() as u64).checked_mul(
                    std::mem::size_of::<(usize, VerifiedOrderedReleasedHeadReplayV14)>() as u64,
                )?)
            })
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>() as u64))
            .ok_or(Denial::BoundExceeded)?;
        if release_count != attachments.len()
            || final_max as u64 > maximum_entries
            || retained_bytes > maximum_retained_bytes
        {
            return Err(Denial::BoundExceeded);
        }
        let (mut checkpoint_copy, mut effective) = super::retained_storage::reserve_rosters(
            source_heads.len(),
            final_max,
            maximum_additional_resident_bytes,
        )?;
        checkpoint_copy.extend_from_slice(source_heads);
        effective.extend_from_slice(source_heads);
        let mut current_ref = checkpoint_ref;
        let mut current_frontier = checkpoint_frontier;
        for (_, attachment) in &attachments {
            let replay = attachment.replay();
            if replay.effect().source_root() != current_ref
                || replay.effect().source_next_block() != current_frontier
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
            apply_entry(&mut effective, expected_prior, next, maximum_entries)?;
            current_ref = Some(replay.result_root());
            current_frontier = replay.result_next_block();
        }
        let effective_root = current_ref.ok_or(Denial::Source)?;
        if selected.release_custody_head_root() != Some(effective_root)
            || selected.next_release_custody_head_block() != current_frontier
        {
            return Err(Denial::Source);
        }
        let mut digest =
            ReleaseCustodyHeadRosterDigestV1::new(Some(effective_root), maximum_entries);
        for entry in &effective {
            digest.push(*entry).map_err(|_| Denial::Mutation)?;
        }
        let (_, effective_digest) = digest.finish();
        Ok(Self {
            checkpoint_source_heads: checkpoint_copy,
            effective_heads: effective,
            checkpoint_source_root: checkpoint_ref,
            checkpoint_source_next_block: checkpoint_frontier,
            effective_root,
            effective_next_block: current_frontier,
            effective_digest,
            effective_root_frame_sha256: claim.selected_root_frame_sha256(),
            ordered_replays: attachments,
            retained_bytes,
        })
    }
}
