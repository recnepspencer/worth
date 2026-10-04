//! Checkpoint-source head custody for a released-generation continuation.
//! The checkpoint attests the rooted inventory of its source root, not a
//! lifetime list of deleted manifests. Present routes must still pass the
//! graph byte reads.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, verify_release_custody_head_controls, BlobReclaimDescriptorV2,
    BlobReclaimSourceBasisV1, BlobRecordKind, BlobRecordV1, PersistedRecordIdentity,
    ReleaseCustodyHeadControlIdentityV1, ReleasedGenerationReclaimBasisV1,
};

use super::super::historical_publication::{observe_all_routes_of_root, RootRouteInventory};
use super::head_predecessor::{checkpoint_head, extends_checkpoint_head};
use super::{binding_matches, selected_blob_record};
use crate::orchestration::planning::{
    context::PlanningContext, resolved_basis::ResolvedPlanningBasis,
};

/// Minted only after the joined checkpoint head of the object is matched to
/// the inventory of the exact checkpoint source root and to its control
/// closure there. It is not a count-based permission to forgive missing
/// bytes or the inventory of an arbitrary later root.
pub(super) struct VerifiedCheckpointSourceAbsence {
    inventory: RootRouteInventory,
}

impl VerifiedCheckpointSourceAbsence {
    /// Absence at the checkpoint source root, which head custody settles.
    pub(super) fn absent_at_source(&self, record: PersistedRecordIdentity) -> bool {
        self.inventory.lacks(record)
    }

    #[cfg(test)]
    pub(super) fn for_test(inventory: RootRouteInventory) -> Self {
        Self { inventory }
    }
}

/// `descriptor` is the batch that extends the checkpoint-source head of
/// `source`. Absence is judged against the inventory read here from the exact
/// checkpoint source root frame, whichever later root the batch was built on.
pub(super) fn authenticate(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    descriptor: BlobReclaimDescriptorV2,
    source: ReleasedGenerationReclaimBasisV1,
    current_count: u16,
) -> Result<(PlanningContext, VerifiedCheckpointSourceAbsence), crate::entry::PhysicalRecoveryOutcome>
{
    let selected = context.selection.root().selected().manifest();
    let selected_sha256: [u8; 32] =
        Sha256::digest(selected.encode(context.authority.record_format)).into();
    let crate::progression::PlanningCustody::SourceHeads(custody) = &basis.custody else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let Some(head) = checkpoint_head(custody.selected_heads(), source) else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let source_root_generation = custody.checkpoint_source_root().generation();
    let source_root_sha256 = custody.source_root_sha256();
    if custody.selected_root() != selected
        || custody.selected_root_sha256() != selected_sha256
        || !extends_checkpoint_head(head, descriptor, current_count, source_root_generation)
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let (next, inventory) = observe_all_routes_of_root(
        context,
        basis,
        source_root_generation,
        head.descriptor_record(),
        source_root_sha256,
    )?;
    context = next;
    if !inventory.lacks(source.publication_record()) {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let (next, bytes) = selected_blob_record(
        context,
        basis,
        source_root_generation,
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
        || prior.base().candidate_root_generation() > source_root_generation
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let (next, manifest_bytes) = selected_blob_record(
        context,
        basis,
        source_root_generation,
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
        || manifest
            .dropped()
            .iter()
            .any(|record| !inventory.lacks(*record))
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let (next, reservation_bytes) = selected_blob_record(
        context,
        basis,
        source_root_generation,
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
    Ok((context, VerifiedCheckpointSourceAbsence { inventory }))
}
