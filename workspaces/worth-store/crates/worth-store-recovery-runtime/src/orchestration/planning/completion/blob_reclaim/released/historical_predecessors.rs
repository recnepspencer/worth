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
use crate::orchestration::planning::page_observation::OrderedReleasedObservation;

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
pub(super) fn authenticate_chain(
    current: BlobReclaimDescriptorV2,
    source: ReleasedGenerationReclaimBasisV1,
    current_count: u16,
    current_prior: Option<ReleaseCustodyHeadEntryV1>,
    ordered: &[OrderedReleasedObservation],
    checkpoint_heads: &[ReleaseCustodyHeadEntryV1],
    maximum_entries: u64,
) -> Option<AuthenticatedPredecessors> {
    let head = checkpoint_head(checkpoint_heads, source);
    let mut predecessor = current.predecessor()?;
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
            let head = head?;
            if !head_predecessor_matches(head, later, later_count) || later_prior != Some(head) {
                return None;
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
            return None;
        }
        let BlobReclaimSourceBasisV1::ReleasedGeneration(prior_source) =
            prior.manifest.source_basis()
        else {
            return None;
        };
        if prior_source.object() != source.object()
            || prior_source.generation() != source.generation()
            || prior_source.publication_record() != source.publication_record()
            || prior.manifest.source_basis_digest() != current.source_basis_digest()
            || prior.manifest.count() == 0
        {
            return None;
        }
        let next_count = (dropped.len() as u64).checked_add(u64::from(prior.manifest.count()))?;
        if next_count > maximum_entries {
            return None;
        }
        dropped
            .try_reserve_exact(prior.manifest.count() as usize)
            .ok()?;
        dropped.extend_from_slice(prior.manifest.dropped());
        ancestors.try_reserve(1).ok()?;
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
        Some(_) if publication_retained => return None,
        Some(_) => head?.cumulative_dropped(),
        None if !publication_retained || later.cumulative_dropped() != u64::from(later_count) => {
            return None
        }
        None => 0,
    };
    if dropped.windows(2).any(|pair| pair[0] == pair[1])
        || base_cumulative
            .checked_add(retained)
            .and_then(|count| count.checked_add(u64::from(current_count)))
            != Some(current.cumulative_dropped())
    {
        return None;
    }
    Some(AuthenticatedPredecessors {
        retained_same_key: dropped,
        anchor,
    })
}
