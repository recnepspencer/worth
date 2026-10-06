//! Prior same-object V3 batches are custody only when their addressed controls
//! and exact ordered root effects have already been verified. Descriptor
//! predecessor bytes alone never authorize a hole in the current closure.
//!
//! A chain that ends at a first release needs no check against the
//! checkpoint heads: the ordered head replay of that release already proved
//! the head tree held no entry for the object at its source root.

use worth_store_physical_format::{
    BlobReclaimDescriptorV2, BlobReclaimSourceBasisV1, PersistedRecordIdentity,
    ReleaseCustodyHeadEntryV1, ReleasedGenerationReclaimBasisV1,
};

use super::head_predecessor::{checkpoint_head, head_predecessor_matches};
use crate::orchestration::planning::completion::historical_publication::HistoricalFailure;
use crate::orchestration::planning::page_observation::{OrderedReleasedObservation, PageLimit};
use crate::orchestration::recovery_budget::RecoveryAllowance;

const INVALID: HistoricalFailure = HistoricalFailure::Invalid;

/// The earliest post-checkpoint batch of the object: the one that extends the
/// checkpoint-source head rather than another ordered batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct HeadAnchoredBatch {
    pub(super) descriptor: BlobReclaimDescriptorV2,
    pub(super) count: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct AuthenticatedPredecessors {
    /// Sorted drops of the object's earlier post-checkpoint batches.
    pub(super) retained_same_key: Vec<PersistedRecordIdentity>,
    /// Present when the chain reaches the checkpoint-source head; absent when
    /// it ends at the object's first release above the checkpoint.
    pub(super) anchor: Option<HeadAnchoredBatch>,
}

/// The head prior that an ordered release replayed for its object.
pub(super) fn replayed_prior(
    release: &OrderedReleasedObservation,
) -> Option<ReleaseCustodyHeadEntryV1> {
    release
        .head_replay
        .replay()
        .effect()
        .mutation()
        .expected_prior()
}

/// Walks the object's earlier post-checkpoint batches. The walk must end
/// either at the object's first release, which drops its publication, or at
/// the checkpoint-source head for the object, which the batch leaving the
/// ordered releases must name, extend and have replayed as its exact prior.
/// The drops it retains are held as one view of no more than `entries`.
pub(super) fn authenticate_chain(
    current: BlobReclaimDescriptorV2,
    source: ReleasedGenerationReclaimBasisV1,
    current_count: u16,
    current_prior: Option<ReleaseCustodyHeadEntryV1>,
    ordered: &[OrderedReleasedObservation],
    checkpoint_heads: &[ReleaseCustodyHeadEntryV1],
    entries: RecoveryAllowance,
) -> Result<AuthenticatedPredecessors, HistoricalFailure> {
    let head = checkpoint_head(checkpoint_heads, source);
    let mut predecessor = current.predecessor().ok_or(INVALID)?;
    let mut later = current;
    let mut later_count = current_count;
    let mut later_prior = current_prior;
    let mut ancestors = Vec::new();
    let mut dropped = Vec::new();
    let anchor = loop {
        let mut matches = ordered.iter().filter(|batch| {
            batch.descriptor_frame.record() == predecessor.descriptor_record()
                && batch.descriptor_frame.payload_sha256() == predecessor.descriptor_frame_sha256()
        });
        let Some(prior) = matches.next() else {
            let head = head.ok_or(INVALID)?;
            if !head_predecessor_matches(head, later, later_count) || later_prior != Some(head) {
                return Err(INVALID);
            }
            break Some(HeadAnchoredBatch {
                descriptor: later,
                count: later_count,
            });
        };
        if matches.next().is_some()
            || ancestors.contains(&prior.descriptor_frame.record())
            || prior.descriptor.encode() != prior.descriptor_frame.bytes()
            || prior.descriptor.base().store() != current.store()
            || prior.descriptor.base().source_basis_digest() != current.source_basis_digest()
            || prior.descriptor.base().terminal()
            || prior.candidate_root_generation > later.source_root_generation()
            || prior.descriptor.base().candidate_root_generation()
                != prior.candidate_root_generation
            || prior
                .descriptor
                .base()
                .cumulative_dropped()
                .checked_add(u64::from(later_count))
                != Some(later.cumulative_dropped())
        {
            return Err(INVALID);
        }
        let BlobReclaimSourceBasisV1::ReleasedGeneration(prior_source) =
            prior.manifest.source_basis()
        else {
            return Err(INVALID);
        };
        if prior_source.object() != source.object()
            || prior_source.generation() != source.generation()
            || prior_source.publication_record() != source.publication_record()
            || prior.manifest.source_basis_digest() != current.source_basis_digest()
            || prior.manifest.count() == 0
        {
            return Err(INVALID);
        }
        retain(dropped.len(), prior.manifest.count(), entries)?;
        dropped
            .try_reserve_exact(prior.manifest.count() as usize)
            .map_err(|_| INVALID)?;
        dropped.extend_from_slice(prior.manifest.dropped());
        ancestors.try_reserve(1).map_err(|_| INVALID)?;
        ancestors.push(prior.descriptor_frame.record());
        later = prior.descriptor.base();
        later_count = prior.manifest.count();
        later_prior = replayed_prior(prior);
        match later.predecessor() {
            Some(older) => predecessor = older,
            None => break None,
        }
    };
    dropped.sort_unstable();
    let retained = dropped.len() as u64;
    let publication_retained = dropped.binary_search(&source.publication_record()).is_ok();
    let base_cumulative = match anchor {
        // The head's own drops predate the checkpoint and removed the
        // publication there; no later batch may drop it again.
        Some(_) if publication_retained => return Err(INVALID),
        Some(_) => head.ok_or(INVALID)?.cumulative_dropped(),
        None if !publication_retained || later.cumulative_dropped() != u64::from(later_count) => {
            return Err(INVALID)
        }
        None => 0,
    };
    if dropped.windows(2).any(|pair| pair[0] == pair[1])
        || base_cumulative
            .checked_add(retained)
            .and_then(|count| count.checked_add(u64::from(current_count)))
            != Some(current.cumulative_dropped())
    {
        return Err(INVALID);
    }
    Ok(AuthenticatedPredecessors {
        retained_same_key: dropped,
        anchor,
    })
}

/// Admits `count` more retained drops beside the `held` ones. Past `entries`
/// is that limit: the same history fits under a wider one.
fn retain(held: usize, count: u16, entries: RecoveryAllowance) -> Result<u64, HistoricalFailure> {
    let needed = (held as u64)
        .checked_add(u64::from(count))
        .ok_or(HistoricalFailure::CountOverflow)?;
    entries
        .admit(needed)
        .map_err(|limit| HistoricalFailure::Limit(PageLimit::Recovery(limit)))
}

#[cfg(test)]
mod retain_tests {
    use super::*;
    use crate::entry::PhysicalRecoveryLimitDimension::ManifestEntries;
    use crate::orchestration::recovery_budget::{allowance_for_test, recovery_limit_for_test};

    #[test]
    fn retained_drops_past_the_entries_are_that_limit_and_not_damage() {
        let entries = allowance_for_test(ManifestEntries, 4);
        assert_eq!(retain(2, 2, entries), Ok(4));
        assert_eq!(
            retain(3, 2, entries),
            Err(HistoricalFailure::Limit(PageLimit::Recovery(
                recovery_limit_for_test(ManifestEntries, 5, 4)
            ))),
        );
    }
}
