//! Bounded effective head roster for the one pending release WAL member. The
//! checkpoint-source roster remains immutable; this is a separate post-WAL
//! state that Store must independently replay and rewalk from actual media.

use crate::VerifiedOrderedReleasedHeadReplayV14;
use worth_store_physical_format::{
    ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadMutationV1,
    ReleaseCustodyHeadRosterDigestV1,
};

#[path = "effective_heads/completed.rs"]
mod completed;
#[path = "effective_heads/ordered.rs"]
mod ordered;
#[path = "effective_heads/pending_preparation.rs"]
mod pending_preparation;
#[path = "effective_heads/retained_storage.rs"]
mod retained_storage;

pub(super) use retained_storage::PreparedEffectiveHeadRosterV14;

use super::{PendingReleaseCheckpointBase, VerifiedPendingWalReleaseCustody};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectiveReleaseHeadDenial {
    MissingReplay,
    Source,
    Mutation,
    BoundExceeded,
    ResidentBoundExceeded { required: u64, admitted: u64 },
}

/// Private-field semantic fold of an exact C.9-admitted pending release member.
/// Its digest does not substitute for a rooted selected-media walk.
#[derive(Debug)]
pub struct VerifiedEffectiveReleaseHeadRosterV14 {
    checkpoint_source_heads: Vec<ReleaseCustodyHeadEntryV1>,
    effective_heads: Vec<ReleaseCustodyHeadEntryV1>,
    checkpoint_source_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
    checkpoint_source_next_block: u64,
    effective_root: ReleaseCustodyHeadBlockReferenceV1,
    effective_next_block: u64,
    effective_digest: [u8; 32],
    effective_root_frame_sha256: [u8; 32],
    ordered_replays: Vec<(usize, VerifiedOrderedReleasedHeadReplayV14)>,
    retained_bytes: u64,
}

impl VerifiedEffectiveReleaseHeadRosterV14 {
    /// The claim already binds its replay to the immutable plan, selected
    /// source path, exact WAL member digest/fate and pending LSN interval.
    pub fn admit_pending(
        claim: &mut VerifiedPendingWalReleaseCustody,
        maximum_entries: u64,
        maximum_retained_bytes: u64,
    ) -> Result<Self, EffectiveReleaseHeadDenial> {
        use EffectiveReleaseHeadDenial as Denial;
        if claim.verified_transition.is_none()
            || claim.ordered_history.is_some()
            || !claim.historical_batches.is_empty()
            || !claim.ordered_released_batches.is_empty()
        {
            return Err(Denial::Source);
        }
        let replay = claim
            .selected_head_replay
            .as_ref()
            .ok_or(Denial::MissingReplay)?;
        let published = claim.published_root.as_ref().ok_or(Denial::Source)?;
        let (source_heads, checkpoint_source_root, checkpoint_source_next_block) = match &claim.base
        {
            PendingReleaseCheckpointBase::NoRelease(_) => {
                if claim.source_root.release_custody_head_root().is_some()
                    || claim.source_root.next_release_custody_head_block() != 1
                {
                    return Err(Denial::Source);
                }
                (&[][..], None, 1)
            }
            PendingReleaseCheckpointBase::ReleasedHeadV2(base) => {
                if base.selected_root() != &claim.source_root
                    || base.selected_root().release_custody_head_root()
                        != base.checkpoint_source_root().release_custody_head_root()
                    || base.selected_root().next_release_custody_head_block()
                        != base
                            .checkpoint_source_root()
                            .next_release_custody_head_block()
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
        if source_heads.len() as u64 > maximum_entries
            || replay.effect().source_root() != claim.source_root.release_custody_head_root()
            || replay.effect().source_root() != checkpoint_source_root
            || replay.effect().source_next_block()
                != claim.source_root.next_release_custody_head_block()
            || published.release_custody_head_root() != Some(replay.result_root())
            || published.next_release_custody_head_block() != replay.result_next_block()
            || replay.operation() != claim.descriptor.custody().request().idempotency()
            || replay.group() != claim.member_group
            || replay.canonical_redo_sha256() != claim.member_redo_digest
            || replay.fate() != claim.operation_fate
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
        if next.descriptor_record() != claim.descriptor_record
            || next.descriptor_frame_sha256() != claim.descriptor_frame_sha256
            || next.manifest_record() != claim.manifest_record
            || next.manifest_frame_sha256() != claim.manifest_frame_sha256
            || next.reservation_record() != claim.reservation_record
            || next.reservation_frame_sha256() != claim.reservation_frame_sha256
        {
            return Err(Denial::Mutation);
        }
        let result_count = source_heads
            .len()
            .checked_add(usize::from(expected_prior.is_none()))
            .ok_or(Denial::BoundExceeded)?;
        let retained_bytes = (source_heads.len() as u64)
            .checked_add(result_count as u64)
            .and_then(|count| {
                count.checked_mul(std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64)
            })
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>() as u64))
            .ok_or(Denial::BoundExceeded)?;
        if result_count as u64 > maximum_entries || retained_bytes > maximum_retained_bytes {
            return Err(Denial::BoundExceeded);
        }
        let prepared = claim
            .prepared_effective_heads
            .as_ref()
            .ok_or(Denial::Source)?;
        if prepared.checkpoint_source_heads != source_heads
            || prepared.effective_heads.len() != result_count
            || !prepared.ordered_replays.is_empty()
            || source_heads
                .windows(2)
                .any(|pair| pair[0].key() >= pair[1].key())
        {
            return Err(Denial::Mutation);
        }
        let index = source_heads.binary_search_by_key(&next.key(), |entry| entry.key());
        if index.ok().map(|at| source_heads[at]) != expected_prior {
            return Err(Denial::Mutation);
        }
        let matches_fold = match index {
            Ok(at) => {
                prepared.effective_heads[..at] == source_heads[..at]
                    && prepared.effective_heads[at] == next
                    && prepared.effective_heads[at + 1..] == source_heads[at + 1..]
            }
            Err(at) => {
                prepared.effective_heads[..at] == source_heads[..at]
                    && prepared.effective_heads[at] == next
                    && prepared.effective_heads[at + 1..] == source_heads[at..]
            }
        };
        if !matches_fold {
            return Err(Denial::Mutation);
        }
        let mut digest =
            ReleaseCustodyHeadRosterDigestV1::new(Some(replay.result_root()), maximum_entries);
        for entry in &prepared.effective_heads {
            digest.push(*entry).map_err(|_| Denial::Mutation)?;
        }
        let (_, effective_digest) = digest.finish();
        let effective_root = replay.result_root();
        let effective_next_block = replay.result_next_block();
        let effective_root_frame_sha256 = claim.published_root_sha256.ok_or(Denial::Source)?;
        let prepared = claim
            .prepared_effective_heads
            .take()
            .ok_or(Denial::Source)?;
        Ok(Self {
            checkpoint_source_heads: prepared.checkpoint_source_heads,
            effective_heads: prepared.effective_heads,
            checkpoint_source_root,
            checkpoint_source_next_block,
            effective_root,
            effective_next_block,
            effective_digest,
            effective_root_frame_sha256,
            ordered_replays: Vec::new(),
            retained_bytes,
        })
    }

    pub fn checkpoint_source_heads(&self) -> &[ReleaseCustodyHeadEntryV1] {
        &self.checkpoint_source_heads
    }
    pub fn effective_heads(&self) -> &[ReleaseCustodyHeadEntryV1] {
        &self.effective_heads
    }
    pub const fn checkpoint_source_root(&self) -> Option<ReleaseCustodyHeadBlockReferenceV1> {
        self.checkpoint_source_root
    }
    pub const fn checkpoint_source_next_block(&self) -> u64 {
        self.checkpoint_source_next_block
    }
    pub const fn effective_root(&self) -> ReleaseCustodyHeadBlockReferenceV1 {
        self.effective_root
    }
    pub const fn effective_next_block(&self) -> u64 {
        self.effective_next_block
    }
    pub const fn effective_digest(&self) -> [u8; 32] {
        self.effective_digest
    }
    pub const fn effective_root_frame_sha256(&self) -> [u8; 32] {
        self.effective_root_frame_sha256
    }
    pub fn ordered_replays(&self) -> &[(usize, VerifiedOrderedReleasedHeadReplayV14)] {
        &self.ordered_replays
    }
    pub const fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }

    /// Owned heap only. Attached replays are charged by the caller before
    /// construction; this reports their actual backing after ownership moves.
    pub fn head_entries_heap_bytes(&self) -> Option<u64> {
        retained_storage::roster_backing_bytes(&self.checkpoint_source_heads, &self.effective_heads)
    }

    pub fn owned_heap_bytes(&self) -> Option<u64> {
        let attachment_bytes =
            (self.ordered_replays.capacity() as u64)
                .checked_mul(
                    std::mem::size_of::<(usize, VerifiedOrderedReleasedHeadReplayV14)>() as u64,
                )?;
        self.ordered_replays.iter().try_fold(
            self.head_entries_heap_bytes()?
                .checked_add(attachment_bytes)?,
            |bytes, (_, replay)| bytes.checked_add(replay.owned_heap_bytes()?),
        )
    }
}

fn fold_entries(
    source: &[ReleaseCustodyHeadEntryV1],
    expected_prior: Option<ReleaseCustodyHeadEntryV1>,
    next: ReleaseCustodyHeadEntryV1,
    result_count: usize,
    mut result: Vec<ReleaseCustodyHeadEntryV1>,
) -> Result<Vec<ReleaseCustodyHeadEntryV1>, EffectiveReleaseHeadDenial> {
    use EffectiveReleaseHeadDenial as Denial;
    let index = source.binary_search_by_key(&next.key(), |entry| entry.key());
    if index.ok().map(|at| source[at]) != expected_prior
        || source.windows(2).any(|pair| pair[0].key() >= pair[1].key())
        || source.len().checked_add(usize::from(index.is_err())) != Some(result_count)
        || !result.is_empty()
        || result.capacity() < result_count
    {
        return Err(Denial::Mutation);
    }
    result.extend_from_slice(source);
    match index {
        Ok(at) => result[at] = next,
        Err(at) => result.insert(at, next),
    }
    Ok(result)
}

fn apply_entry(
    entries: &mut Vec<ReleaseCustodyHeadEntryV1>,
    expected_prior: Option<ReleaseCustodyHeadEntryV1>,
    next: ReleaseCustodyHeadEntryV1,
    maximum_entries: u64,
) -> Result<(), EffectiveReleaseHeadDenial> {
    use EffectiveReleaseHeadDenial as Denial;
    let index = entries.binary_search_by_key(&next.key(), |entry| entry.key());
    if index.ok().map(|at| entries[at]) != expected_prior {
        return Err(Denial::Mutation);
    }
    match index {
        Ok(at) => entries[at] = next,
        Err(at) => {
            if entries.len() as u64 >= maximum_entries {
                return Err(Denial::BoundExceeded);
            }
            if entries.len() >= entries.capacity() {
                return Err(Denial::BoundExceeded);
            }
            entries.insert(at, next);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use worth_store_physical_format::{PersistedRecordIdentity, ReleaseCustodyHeadKeyV1};

    use super::*;

    fn entry(object: u8, descriptor: u64) -> ReleaseCustodyHeadEntryV1 {
        let record = |ordinal| PersistedRecordIdentity::new([8; 16], ordinal).unwrap();
        ReleaseCustodyHeadEntryV1::new(
            ReleaseCustodyHeadKeyV1::new([object; 16], 1).unwrap(),
            record(descriptor),
            [1; 32],
            record(descriptor + 1),
            [2; 32],
            record(descriptor + 2),
            [3; 32],
            [4; 32],
            None,
            4,
            1,
            false,
        )
        .unwrap()
    }

    #[test]
    fn pending_head_fold_rejects_wrong_prior_or_key_and_preserves_other_heads() {
        let first = entry(1, 1);
        let unrelated = entry(3, 10);
        let replacement = entry(1, 20);
        let source = [first, unrelated];
        assert_eq!(
            fold_entries(&source, Some(first), replacement, 2, Vec::with_capacity(2)).unwrap(),
            vec![replacement, unrelated]
        );
        assert_eq!(
            fold_entries(
                &source,
                Some(unrelated),
                replacement,
                2,
                Vec::with_capacity(2)
            ),
            Err(EffectiveReleaseHeadDenial::Mutation)
        );
        assert_eq!(
            fold_entries(&source, None, replacement, 3, Vec::with_capacity(3)),
            Err(EffectiveReleaseHeadDenial::Mutation)
        );
        assert_eq!(
            fold_entries(&source, Some(first), entry(2, 30), 2, Vec::with_capacity(2)),
            Err(EffectiveReleaseHeadDenial::Mutation)
        );
        assert_eq!(
            fold_entries(&source, None, entry(2, 30), 3, Vec::with_capacity(3)).unwrap(),
            vec![first, entry(2, 30), unrelated]
        );

        let mut bounded = Vec::with_capacity(3);
        bounded.extend_from_slice(&source);
        assert_eq!(
            apply_entry(&mut bounded, None, entry(2, 30), 2),
            Err(EffectiveReleaseHeadDenial::BoundExceeded)
        );
        assert_eq!(bounded, source.to_vec());
        assert_eq!(
            apply_entry(&mut bounded, Some(entry(2, 30)), entry(2, 31), 3),
            Err(EffectiveReleaseHeadDenial::Mutation)
        );
        assert_eq!(
            apply_entry(&mut bounded, None, replacement, 3),
            Err(EffectiveReleaseHeadDenial::Mutation)
        );
        assert_eq!(bounded, source.to_vec());
    }
}
