//! Current keyed custody for a released-generation continuation.
//! The checkpoint attests the rooted selected inventory, not a lifetime list
//! of deleted manifests. Present routes must still pass the graph byte reads.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, verify_release_custody_head_controls, BlobReclaimDescriptorV2,
    BlobReclaimSourceBasisV1, BlobRecordKind, BlobRecordV1, DurablePhysicalRootManifest,
    PhysicalRecordFormatDeclaration, ReleaseCustodyHeadControlIdentityV1, ReleaseCustodyHeadKeyV1,
    ReleasedGenerationReclaimBasisV1,
};

use super::{binding_matches, selected_blob_record};
use crate::orchestration::planning::{
    context::PlanningContext, resolved_basis::ResolvedPlanningBasis,
};

/// Minted only after joined checkpoint custody is matched to the exact
/// selected root and its current control closure. It is not a count-based
/// permission to forgive missing bytes or an arbitrary later route inventory.
pub(super) struct VerifiedSelectedResidualAbsence {
    source_root_sha256: [u8; 32],
    source_basis_digest: [u8; 32],
}

impl VerifiedSelectedResidualAbsence {
    pub(super) fn matches_source(
        &self,
        root: &DurablePhysicalRootManifest,
        format: PhysicalRecordFormatDeclaration,
        source: ReleasedGenerationReclaimBasisV1,
    ) -> bool {
        self.source_root_sha256 == <[u8; 32]>::from(Sha256::digest(root.encode(format)))
            && self.source_basis_digest
                == BlobReclaimSourceBasisV1::ReleasedGeneration(source)
                    .digest(source.publication().store())
    }
}

pub(super) fn authenticate(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    descriptor: BlobReclaimDescriptorV2,
    source: ReleasedGenerationReclaimBasisV1,
    current_count: u16,
) -> Result<(PlanningContext, VerifiedSelectedResidualAbsence), crate::entry::PhysicalRecoveryOutcome>
{
    let selected = context.selection.root().selected().manifest();
    let selected_sha256: [u8; 32] =
        Sha256::digest(selected.encode(context.authority.record_format)).into();
    let crate::progression::PlanningCustody::SourceHeads(custody) = &basis.custody else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let Some(key) = ReleaseCustodyHeadKeyV1::new(source.object(), source.generation()) else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let head = custody
        .selected_heads()
        .binary_search_by_key(&key, |entry| entry.key())
        .ok()
        .and_then(|index| custody.selected_heads().get(index))
        .copied();
    let Some(head) = head else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    if custody.selected_root() != selected
        || custody.selected_root_sha256() != selected_sha256
        || custody.checkpoint_source_root().release_custody_head_root()
            != selected.release_custody_head_root()
        || custody
            .checkpoint_source_root()
            .next_release_custody_head_block()
            != selected.next_release_custody_head_block()
        || descriptor.source_root_generation() != selected.generation()
        || descriptor.predecessor().is_none_or(|prior| {
            prior.descriptor_record() != head.descriptor_record()
                || prior.descriptor_frame_sha256() != head.descriptor_frame_sha256()
        })
        || head.terminal()
        || head.source_basis_digest() != descriptor.source_basis_digest()
        || head
            .cumulative_dropped()
            .checked_add(u64::from(current_count))
            != Some(descriptor.cumulative_dropped())
        || context
            .selection
            .page_facts()
            .placements()
            .iter()
            .any(|route| route.record() == source.publication_record())
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let (next, bytes) = selected_blob_record(
        context,
        basis,
        descriptor.source_root_generation(),
        head.descriptor_record(),
        BlobRecordKind::ReclaimDescriptorV3,
    )?;
    context = next;
    let Ok(BlobRecordV1::ReclaimDescriptorV3(prior)) = decode_blob_record(&bytes) else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    if <[u8; 32]>::from(Sha256::digest(&bytes)) != head.descriptor_frame_sha256()
        || prior.base().store() != descriptor.store()
        || prior.base().source_basis_digest() != descriptor.source_basis_digest()
        || prior.base().cumulative_dropped() != head.cumulative_dropped()
        || prior.base().terminal() != head.terminal()
        || prior.base().candidate_root_generation() > descriptor.source_root_generation()
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let (next, manifest_bytes) = selected_blob_record(
        context,
        basis,
        descriptor.source_root_generation(),
        head.manifest_record(),
        BlobRecordKind::DropSetManifestV3,
    )?;
    context = next;
    let Ok(BlobRecordV1::DropSetManifestV3(manifest)) = decode_blob_record(&manifest_bytes) else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    if !binding_matches(
        prior.base(),
        &manifest,
        &manifest_bytes,
        head.descriptor_record(),
    ) || manifest.source_basis() != BlobReclaimSourceBasisV1::ReleasedGeneration(source)
        || prior.base().manifest_record() != head.manifest_record()
        || prior.base().manifest_frame_sha256() != head.manifest_frame_sha256()
        || prior.base().predecessor() != head.predecessor()
        || manifest.dropped().iter().any(|record| {
            context
                .selection
                .page_facts()
                .placements()
                .iter()
                .any(|route| route.record() == *record)
        })
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let (next, reservation_bytes) = selected_blob_record(
        context,
        basis,
        descriptor.source_root_generation(),
        head.reservation_record(),
        BlobRecordKind::OriginalDropReserved,
    )?;
    context = next;
    let Ok(BlobRecordV1::OriginalDropReserved(reservation)) =
        decode_blob_record(&reservation_bytes)
    else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let frames = ReleaseCustodyHeadControlIdentityV1::new(
        head.descriptor_record(),
        Sha256::digest(&bytes).into(),
        head.reservation_record(),
        Sha256::digest(&reservation_bytes).into(),
        head.manifest_record(),
        Sha256::digest(&manifest_bytes).into(),
    );
    if frames
        .and_then(|frames| {
            verify_release_custody_head_controls(head, frames, prior, reservation, &manifest)
        })
        .is_err()
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    Ok((
        context,
        VerifiedSelectedResidualAbsence {
            source_root_sha256: selected_sha256,
            source_basis_digest: descriptor.source_basis_digest(),
        },
    ))
}
