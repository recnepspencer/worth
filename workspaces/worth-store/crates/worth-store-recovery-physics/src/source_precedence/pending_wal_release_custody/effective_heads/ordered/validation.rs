//! Allocation-free C.9/history binding shared by preparation and final seal.

use worth_store_physical_format::{
    ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadMutationV1,
    ReleaseCustodyHeadNodeWriteV1, ReleaseCustodyHeadPathNodeV1,
};

use super::super::{
    EffectiveReleaseHeadDenial as Denial, PendingReleaseCheckpointBase,
    PreparedEffectiveHeadRosterV14, VerifiedPendingWalReleaseCustody,
};
use crate::{VerifiedOrderedReleasedHeadReplayV14, VerifiedOrderedRootEdge};

pub(super) struct OrderedBasis<'a> {
    pub(super) source_heads: &'a [ReleaseCustodyHeadEntryV1],
    pub(super) checkpoint_ref: Option<ReleaseCustodyHeadBlockReferenceV1>,
    pub(super) checkpoint_frontier: u64,
    final_max: usize,
    effect_bytes: u64,
}

impl OrderedBasis<'_> {
    pub(super) fn retained_bytes<T>(
        &self,
        attachments: &Vec<(usize, VerifiedOrderedReleasedHeadReplayV14)>,
    ) -> Result<u64, Denial> {
        (self.source_heads.len() as u64)
            .checked_add(self.final_max as u64)
            .and_then(|entries| {
                entries.checked_mul(std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64)
            })
            .and_then(|bytes| bytes.checked_add(self.effect_bytes))
            .and_then(|bytes| {
                bytes.checked_add((attachments.capacity() as u64).checked_mul(
                    std::mem::size_of::<(usize, VerifiedOrderedReleasedHeadReplayV14)>() as u64,
                )?)
            })
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<T>() as u64))
            .ok_or(Denial::BoundExceeded)
    }

    pub(super) const fn result_capacity(&self) -> usize {
        self.final_max
    }
}

pub(super) fn validate<'a>(
    claim: &'a VerifiedPendingWalReleaseCustody,
    attachments: &[(usize, VerifiedOrderedReleasedHeadReplayV14)],
    maximum_entries: u64,
) -> Result<OrderedBasis<'a>, Denial> {
    let history = claim.ordered_history.as_ref().ok_or(Denial::Source)?;
    let pending = claim
        .selected_head_replay
        .as_ref()
        .ok_or(Denial::MissingReplay)?;
    let (source_heads, checkpoint_ref, checkpoint_frontier) = match &claim.base {
        PendingReleaseCheckpointBase::NoRelease(_) => (&[][..], None, 1),
        PendingReleaseCheckpointBase::ReleasedHeadV2(base) => {
            if base.source_root_sha256() != claim.checkpoint_source_root_sha256
                || base.selected_root() != &claim.source_root
                || base.selected_root_sha256() != claim.source_root_sha256
            {
                return Err(Denial::Source);
            }
            (
                base.selected_heads(),
                base.checkpoint_source_root().release_custody_head_root(),
                base.checkpoint_source_root()
                    .next_release_custody_head_block(),
            )
        }
        _ => return Err(Denial::Source),
    };
    if history.checkpoint_root_frame_sha256() != claim.checkpoint_source_root_sha256
        || history.selected_root_frame_sha256() != claim.source_root_sha256
        || claim.ordered_released_batches.len() != attachments.len()
        || !claim.historical_batches.is_empty()
        || attachments.is_empty()
        || maximum_entries == 0
        || source_heads.len() as u64 > maximum_entries
        || source_heads
            .windows(2)
            .any(|pair| pair[0].key() >= pair[1].key())
    {
        return Err(Denial::Source);
    }
    let mut released_count = 0usize;
    let mut insert_count = 0usize;
    let mut effect_bytes = 0u64;
    let mut current_ref = checkpoint_ref;
    let mut current_frontier = checkpoint_frontier;
    for (index, edge) in history.edges().iter().enumerate() {
        let VerifiedOrderedRootEdge::Released(released) = edge else {
            continue;
        };
        let (attached_index, attachment) = attachments.get(released_count).ok_or(Denial::Source)?;
        let batch = claim
            .ordered_released_batches
            .get(released_count)
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
            || replay.effect().source_root() != current_ref
            || replay.effect().source_next_block() != current_frontier
            || next.descriptor_record() != batch.descriptor_frame().record()
            || next.descriptor_frame_sha256() != batch.descriptor_frame().payload_sha256()
            || next.manifest_record() != batch.manifest_frame().record()
            || next.manifest_frame_sha256() != batch.manifest_frame().payload_sha256()
            || next.reservation_record() != batch.reservation_frame().record()
            || next.reservation_frame_sha256() != batch.reservation_frame().payload_sha256()
        {
            return Err(Denial::Mutation);
        }
        insert_count = insert_count
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
                (replay.source_path().len() as u64)
                    .checked_mul(std::mem::size_of::<ReleaseCustodyHeadPathNodeV1>() as u64)
                    .and_then(|added| bytes.checked_add(added))
            })
            .and_then(|bytes| {
                (replay.node_writes().len() as u64)
                    .checked_mul(std::mem::size_of::<ReleaseCustodyHeadNodeWriteV1>() as u64)
                    .and_then(|added| bytes.checked_add(added))
            })
            .ok_or(Denial::BoundExceeded)?;
        current_ref = Some(replay.result_root());
        current_frontier = replay.result_next_block();
        released_count += 1;
    }
    let ReleaseCustodyHeadMutationV1::Upsert {
        expected_prior,
        next,
    } = pending.effect().mutation()
    else {
        return Err(Denial::Mutation);
    };
    insert_count = insert_count
        .checked_add(usize::from(expected_prior.is_none()))
        .ok_or(Denial::BoundExceeded)?;
    let final_max = source_heads
        .len()
        .checked_add(insert_count)
        .ok_or(Denial::BoundExceeded)?;
    if released_count != attachments.len()
        || final_max as u64 > maximum_entries
        || current_ref != claim.source_root.release_custody_head_root()
        || current_frontier != claim.source_root.next_release_custody_head_block()
        || pending.effect().source_root() != current_ref
        || pending.effect().source_next_block() != current_frontier
        || pending.operation() != claim.descriptor.custody().request().idempotency()
        || pending.group() != claim.member_group
        || pending.canonical_redo_sha256() != claim.member_redo_digest
        || pending.fate() != claim.operation_fate
        || next.descriptor_record() != claim.descriptor_record
        || next.descriptor_frame_sha256() != claim.descriptor_frame_sha256
        || next.manifest_record() != claim.manifest_record
        || next.manifest_frame_sha256() != claim.manifest_frame_sha256
        || next.reservation_record() != claim.reservation_record
        || next.reservation_frame_sha256() != claim.reservation_frame_sha256
    {
        return Err(Denial::Source);
    }
    Ok(OrderedBasis {
        source_heads,
        checkpoint_ref,
        checkpoint_frontier,
        final_max,
        effect_bytes,
    })
}

/// Check the prepared result against every immutable keyed mutation without
/// allocating after publication. The sorted result must contain precisely
/// the source keys and all introduced keys, with each prior matching.
pub(super) fn matches_exact_fold(
    claim: &VerifiedPendingWalReleaseCustody,
    prepared: &PreparedEffectiveHeadRosterV14,
    source: &[ReleaseCustodyHeadEntryV1],
) -> bool {
    let result = &prepared.effective_heads;
    if result.windows(2).any(|pair| pair[0].key() >= pair[1].key())
        || source.iter().any(|entry| {
            result
                .binary_search_by_key(&entry.key(), |other| other.key())
                .is_err()
        })
    {
        return false;
    }
    let Some(pending) = claim.selected_head_replay.as_ref() else {
        return false;
    };
    for (_, attachment) in &prepared.ordered_replays {
        let ReleaseCustodyHeadMutationV1::Upsert { next, .. } =
            attachment.replay().effect().mutation()
        else {
            return false;
        };
        if result
            .binary_search_by_key(&next.key(), |entry| entry.key())
            .is_err()
        {
            return false;
        }
    }
    let ReleaseCustodyHeadMutationV1::Upsert { next, .. } = pending.effect().mutation() else {
        return false;
    };
    if result
        .binary_search_by_key(&next.key(), |entry| entry.key())
        .is_err()
    {
        return false;
    }
    for entry in result {
        let mut current = source
            .binary_search_by_key(&entry.key(), |other| other.key())
            .ok()
            .map(|index| source[index]);
        for (_, attachment) in &prepared.ordered_replays {
            let ReleaseCustodyHeadMutationV1::Upsert {
                expected_prior,
                next,
            } = attachment.replay().effect().mutation()
            else {
                return false;
            };
            if next.key() == entry.key() {
                if current != expected_prior {
                    return false;
                }
                current = Some(next);
            }
        }
        let ReleaseCustodyHeadMutationV1::Upsert {
            expected_prior,
            next,
        } = pending.effect().mutation()
        else {
            return false;
        };
        if next.key() == entry.key() {
            if current != expected_prior {
                return false;
            }
            current = Some(next);
        }
        if current != Some(*entry) {
            return false;
        }
    }
    true
}
