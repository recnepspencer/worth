//! A V3 predecessor is per released object. An earlier same-object batch in
//! the ordered post-checkpoint history is the predecessor when one exists;
//! otherwise the checkpoint-source custody for that object is.

use worth_store_physical_format::{
    BlobReclaimDescriptorV3, BlobReclaimSourceBasisV1, DropSetManifestV3, PersistedRecordIdentity,
    ReleaseCheckpointBatchV1, ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1,
    ReleasedDropPredecessorV1,
};
use worth_store_recovery_physics::{
    VerifiedOrderedPendingWalReleaseBatch, VerifiedPendingWalReleaseCustody,
};

use super::super::control_frames::ObservedSelectedControls;

/// Batch/Accumulator custody of a pending claim that carries no head roster,
/// with the selected controls Store observed for it.
#[derive(Clone, Copy)]
pub(super) struct LegacyBatchBase<'a> {
    pub(super) claim: &'a VerifiedPendingWalReleaseCustody,
    pub(super) controls: Option<&'a ObservedSelectedControls>,
}

pub(super) fn predecessor_matches(
    claim: &VerifiedPendingWalReleaseCustody,
    selected_base: Option<&ObservedSelectedControls>,
    current: BlobReclaimDescriptorV3,
    manifest: &DropSetManifestV3,
) -> bool {
    ordered_or_base_matches(
        claim.ordered_released_batches(),
        claim
            .selected_head_v2()
            .map(|joined| joined.selected_heads()),
        Some(LegacyBatchBase {
            claim,
            controls: selected_base,
        }),
        current,
        manifest,
    )
}

/// The Store-wide edge order is not the per-object predecessor chain. A batch
/// can follow a different object's batch, but a successor for the same source
/// must name the exact prior descriptor frame and advance only that object's
/// cumulative count. The first post-checkpoint batch of an object extends its
/// checkpoint-source custody instead.
pub(super) fn ordered_predecessor_matches(
    prior: &[VerifiedOrderedPendingWalReleaseBatch],
    checkpoint_heads: Option<&[ReleaseCustodyHeadEntryV1]>,
    legacy_base: Option<LegacyBatchBase<'_>>,
    current: &VerifiedOrderedPendingWalReleaseBatch,
) -> bool {
    ordered_or_base_matches(
        prior,
        checkpoint_heads,
        legacy_base,
        current.descriptor(),
        &current.manifest(),
    )
}

fn ordered_or_base_matches(
    prior: &[VerifiedOrderedPendingWalReleaseBatch],
    checkpoint_heads: Option<&[ReleaseCustodyHeadEntryV1]>,
    legacy_base: Option<LegacyBatchBase<'_>>,
    current: BlobReclaimDescriptorV3,
    manifest: &DropSetManifestV3,
) -> bool {
    let source = manifest.source_basis();
    let Some(earlier) = prior
        .iter()
        .rev()
        .find(|earlier| earlier.manifest().source_basis() == source)
    else {
        return selected_base_predecessor_matches(checkpoint_heads, legacy_base, current, manifest);
    };
    current.base().predecessor().is_some_and(|predecessor| {
        earlier.descriptor_frame().record() == predecessor.descriptor_record()
            && earlier.descriptor_frame().payload_sha256() == predecessor.descriptor_frame_sha256()
            && ordered_prior_matches(earlier, current, manifest)
    })
}

/// The one owner of "this V3 extends the checkpoint-source custody of its
/// object". A head roster decides when the checkpoint carries one; a Batch
/// base without heads keeps its addressed-controls proof.
pub(super) fn selected_base_predecessor_matches(
    checkpoint_heads: Option<&[ReleaseCustodyHeadEntryV1]>,
    legacy_base: Option<LegacyBatchBase<'_>>,
    current: BlobReclaimDescriptorV3,
    manifest: &DropSetManifestV3,
) -> bool {
    if let Some(heads) = checkpoint_heads {
        return head_predecessor_matches(heads, current, manifest);
    }
    let Some(legacy) = legacy_base else {
        return false;
    };
    if current.base().predecessor().is_some() {
        return legacy_base_predecessor_matches(legacy.claim, legacy.controls, current, manifest);
    }
    !selected_base_matches_source(legacy.claim, legacy.controls, manifest)
        && current.base().cumulative_dropped() == u64::from(manifest.count())
        && matches!(manifest.source_basis(),
            BlobReclaimSourceBasisV1::ReleasedGeneration(source)
                if manifest.dropped().binary_search(&source.publication_record()).is_ok())
}

/// A head for the object is its exact predecessor: nonterminal, over the same
/// source basis, with the cumulative count advancing by this manifest. With
/// no head the batch is the object's first release and drops its publication.
fn head_predecessor_matches(
    heads: &[ReleaseCustodyHeadEntryV1],
    current: BlobReclaimDescriptorV3,
    manifest: &DropSetManifestV3,
) -> bool {
    let base = current.base();
    let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = manifest.source_basis() else {
        return false;
    };
    let Some(key) = ReleaseCustodyHeadKeyV1::new(source.object(), source.generation()) else {
        return false;
    };
    let prior = heads
        .binary_search_by_key(&key, |entry| entry.key())
        .ok()
        .and_then(|index| heads.get(index))
        .copied();
    match prior {
        Some(head) => {
            !head.terminal()
                && head.source_basis_digest() == base.source_basis_digest()
                && head.source_root_generation() < base.source_root_generation()
                && base.predecessor()
                    == ReleasedDropPredecessorV1::new(
                        head.descriptor_record(),
                        head.descriptor_frame_sha256(),
                    )
                    .ok()
                && head
                    .cumulative_dropped()
                    .checked_add(u64::from(manifest.count()))
                    == Some(base.cumulative_dropped())
                && !manifest.dropped().contains(&source.publication_record())
        }
        None => {
            base.predecessor().is_none()
                && base.cumulative_dropped() == u64::from(manifest.count())
                && manifest.dropped().contains(&source.publication_record())
        }
    }
}

pub(super) fn selected_base_matches_source(
    claim: &VerifiedPendingWalReleaseCustody,
    selected_base: Option<&ObservedSelectedControls>,
    manifest: &DropSetManifestV3,
) -> bool {
    let Some(controls) = selected_base else {
        return false;
    };
    if claim.addressed_release_base().is_some() {
        return controls
            .latest_for_source(manifest.source_basis())
            .is_some();
    }
    let (batches, tip) = match (claim.selected_release(), claim.addressed_release_base()) {
        (Some(base), None) => (base.batches(), base.accumulator().tip()),
        (None, Some(base)) => (base.batches(), base.accumulator().tip()),
        _ => return false,
    };
    batches
        .iter()
        .map(|batch| (batch.descriptor_record(), batch.descriptor_frame_sha256()))
        .chain(std::iter::once((
            tip.descriptor_record(),
            tip.descriptor_frame_sha256(),
        )))
        .any(|(record, sha)| {
            controls
                .predecessor_controls(record, sha)
                .is_some_and(|(_, prior_manifest)| {
                    prior_manifest.source_basis() == manifest.source_basis()
                })
        })
}

fn legacy_base_predecessor_matches(
    claim: &VerifiedPendingWalReleaseCustody,
    selected_base: Option<&ObservedSelectedControls>,
    current: BlobReclaimDescriptorV3,
    manifest: &DropSetManifestV3,
) -> bool {
    let Some(prior_controls) = selected_base else {
        return false;
    };
    if let Some(addressed) = claim.addressed_release_base() {
        let Some(predecessor) = current.base().predecessor() else {
            return false;
        };
        let exact = (
            predecessor.descriptor_record(),
            predecessor.descriptor_frame_sha256(),
        );
        let tip = addressed.accumulator().tip();
        let certified = addressed
            .batches()
            .iter()
            .any(|batch| (batch.descriptor_record(), batch.descriptor_frame_sha256()) == exact)
            || (tip.descriptor_record(), tip.descriptor_frame_sha256()) == exact;
        // Older routed residue rejects a forged fresh chain but does not
        // itself carry durable per-object custody through tag-7 compaction.
        if !certified {
            return false;
        }
        if prior_controls.latest_for_source(manifest.source_basis()) != Some(exact) {
            return false;
        }
        let Some((prior, prior_manifest)) = prior_controls.predecessor_controls(
            predecessor.descriptor_record(),
            predecessor.descriptor_frame_sha256(),
        ) else {
            return false;
        };
        let base = current.base();
        let prior_base = prior.base();
        return prior_base.store() == base.store()
            && !prior_base.terminal()
            && prior_base.candidate_root_generation() <= base.source_root_generation()
            && prior_base.source_basis_digest() == base.source_basis_digest()
            && same_source_and_cumulative(
                prior_manifest.source_basis(),
                manifest.source_basis(),
                prior_base.cumulative_dropped(),
                manifest.count(),
                base.cumulative_dropped(),
            );
    }
    let (batches, tip) = match (claim.selected_release(), claim.addressed_release_base()) {
        (Some(base), None) => (base.batches(), base.accumulator().tip()),
        (None, Some(base)) => (base.batches(), base.accumulator().tip()),
        _ => return false,
    };
    let Some(predecessor) = current.base().predecessor() else {
        return false;
    };
    // The tip is newest globally; older Batch records follow in reverse
    // certificate order. A pointer to an older batch of this same object may
    // not skip a newer drop merely because its SHA still matches.
    let latest_for_object =
        std::iter::once((tip.descriptor_record(), tip.descriptor_frame_sha256()))
            .chain(
                batches
                    .iter()
                    .rev()
                    .map(|batch| (batch.descriptor_record(), batch.descriptor_frame_sha256())),
            )
            .find(|(record, sha)| {
                prior_controls
                    .predecessor_controls(*record, *sha)
                    .is_some_and(|(_, prior_manifest)| {
                        prior_manifest.source_basis() == manifest.source_basis()
                    })
            });
    if latest_for_object
        != Some((
            predecessor.descriptor_record(),
            predecessor.descriptor_frame_sha256(),
        ))
    {
        return false;
    }
    selected_predecessor_matches(
        batches,
        Some((tip.descriptor_record(), tip.descriptor_frame_sha256())),
        current,
        manifest,
        |record, sha| prior_controls.predecessor_controls(record, sha),
    )
}

fn ordered_prior_matches(
    earlier: &VerifiedOrderedPendingWalReleaseBatch,
    current: BlobReclaimDescriptorV3,
    manifest: &DropSetManifestV3,
) -> bool {
    let earlier_base = earlier.descriptor().base();
    let base = current.base();
    !earlier_base.terminal()
        && earlier_base.store() == base.store()
        && earlier_base.candidate_root_generation() <= base.source_root_generation()
        && earlier_base.source_basis_digest() == base.source_basis_digest()
        && same_source_and_cumulative(
            earlier.manifest().source_basis(),
            manifest.source_basis(),
            earlier_base.cumulative_dropped(),
            manifest.count(),
            base.cumulative_dropped(),
        )
}

fn selected_predecessor_matches(
    batches: &[ReleaseCheckpointBatchV1],
    selected_tip: Option<(PersistedRecordIdentity, [u8; 32])>,
    current: BlobReclaimDescriptorV3,
    manifest: &DropSetManifestV3,
    lookup: impl FnOnce(
        PersistedRecordIdentity,
        [u8; 32],
    ) -> Option<(BlobReclaimDescriptorV3, DropSetManifestV3)>,
) -> bool {
    let base = current.base();
    let Some(predecessor) = base.predecessor() else {
        return false;
    };
    let in_batch = batches.iter().any(|batch| {
        batch.descriptor_record() == predecessor.descriptor_record()
            && batch.descriptor_frame_sha256() == predecessor.descriptor_frame_sha256()
    });
    let exact_tip = selected_tip.is_some_and(|(record, sha)| {
        record == predecessor.descriptor_record() && sha == predecessor.descriptor_frame_sha256()
    });
    if !in_batch && !exact_tip {
        return false;
    }
    let Some((prior, prior_manifest)) = lookup(
        predecessor.descriptor_record(),
        predecessor.descriptor_frame_sha256(),
    ) else {
        return false;
    };
    let prior_base = prior.base();
    prior_base.store() == base.store()
        && !prior_base.terminal()
        && prior_base.candidate_root_generation() <= base.source_root_generation()
        && same_source_and_cumulative(
            prior_manifest.source_basis(),
            manifest.source_basis(),
            prior_base.cumulative_dropped(),
            manifest.count(),
            base.cumulative_dropped(),
        )
        && prior_base.source_basis_digest() == base.source_basis_digest()
}

fn same_source_and_cumulative(
    prior: BlobReclaimSourceBasisV1,
    current: BlobReclaimSourceBasisV1,
    prior_dropped: u64,
    current_count: u16,
    current_dropped: u64,
) -> bool {
    matches!((prior, current),
        (BlobReclaimSourceBasisV1::ReleasedGeneration(before),
         BlobReclaimSourceBasisV1::ReleasedGeneration(after)) if before == after)
        && prior_dropped.checked_add(u64::from(current_count)) == Some(current_dropped)
}

#[cfg(test)]
#[path = "lineage/tests.rs"]
mod tests;
