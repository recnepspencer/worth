//! Prior same-object V3 batches are custody only when their addressed controls
//! and exact ordered root effects have already been verified. Descriptor
//! predecessor bytes alone never authorize a hole in the current closure.

use worth_store_physical_format::{
    BlobReclaimDescriptorV2, BlobReclaimSourceBasisV1, PersistedRecordIdentity,
    ReleasedGenerationReclaimBasisV1,
};

use crate::orchestration::planning::page_observation::OrderedReleasedObservation;

/// This variant is valid only for a checkpoint-anchored NoRelease history:
/// the first ancestor must be the first release of the object and drop its
/// publication. A selected Batch base needs its own addressed predecessor.
pub(super) fn authenticate_no_release_chain(
    current: BlobReclaimDescriptorV2,
    source: ReleasedGenerationReclaimBasisV1,
    current_count: u16,
    ordered: &[OrderedReleasedObservation],
    maximum_entries: u64,
) -> Option<Vec<PersistedRecordIdentity>> {
    let mut predecessor = current.predecessor()?;
    let mut later_cumulative = current.cumulative_dropped();
    let mut later_count = current_count;
    let mut later_source_generation = current.source_root_generation();
    let mut ancestors = Vec::new();
    let mut dropped = Vec::new();
    loop {
        let mut matches = ordered.iter().filter(|batch| {
            batch.descriptor_frame.record() == predecessor.descriptor_record()
                && batch.descriptor_frame.payload_sha256() == predecessor.descriptor_frame_sha256()
        });
        let prior = matches.next()?;
        if matches.next().is_some()
            || ancestors.contains(&prior.descriptor_frame.record())
            || prior.descriptor.encode() != prior.descriptor_frame.bytes()
            || prior.descriptor.base().store() != current.store()
            || prior.descriptor.base().source_basis_digest() != current.source_basis_digest()
            || prior.descriptor.base().terminal()
            || prior.candidate_root_generation > later_source_generation
            || prior.descriptor.base().candidate_root_generation()
                != prior.candidate_root_generation
            || prior
                .descriptor
                .base()
                .cumulative_dropped()
                .checked_add(u64::from(later_count))
                != Some(later_cumulative)
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
        later_cumulative = prior.descriptor.base().cumulative_dropped();
        later_count = prior.manifest.count();
        later_source_generation = prior.descriptor.base().source_root_generation();
        if let Some(older) = prior.descriptor.base().predecessor() {
            predecessor = older;
        } else {
            break;
        }
    }
    dropped.sort_unstable();
    if dropped.windows(2).any(|pair| pair[0] == pair[1])
        || dropped.binary_search(&source.publication_record()).is_err()
        || current.cumulative_dropped()
            != (dropped.len() as u64).checked_add(u64::from(current_count))?
        || later_cumulative != u64::from(later_count)
    {
        return None;
    }
    Some(dropped)
}
