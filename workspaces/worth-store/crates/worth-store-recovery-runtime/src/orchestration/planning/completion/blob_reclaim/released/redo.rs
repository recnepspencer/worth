//! Durable released-drop WAL requires selected physical custody before C8
//! may materialize the exact root-changing drop.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobReclaimDescriptorV3, BlobReclaimSourceBasisV1, BlobRecordKind, BlobRecordV1,
    PersistedRecordIdentity,
};

use super::super::super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use super::{
    binding_matches, closure_evidence::ReleasedClosureEvidence, historical_anchor, selected,
    selected_blob_record,
};

pub(in crate::orchestration::planning::completion) fn preflight(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    descriptor: BlobReclaimDescriptorV3,
    descriptor_record: PersistedRecordIdentity,
    operation_id: [u8; 32],
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    let base = descriptor.base();
    let custody = descriptor.custody();
    let selected_generation = context
        .selection
        .root()
        .selected()
        .selector()
        .root_generation();
    let selected_root_digest: [u8; 32] = Sha256::digest(
        context
            .selection
            .root()
            .selected()
            .manifest()
            .encode(context.authority.record_format),
    )
    .into();
    let selected_free_digest: [u8; 32] = Sha256::digest(
        basis
            .observed_pages
            .selected_source
            .free_space
            .encode(context.authority.record_format),
    )
    .into();
    if base.store() != context.authority.media.store_identity().bytes()
        || base.source_root_generation() != selected_generation
        || base.manifest_record() == descriptor_record
        || custody.request().idempotency() != operation_id
        || custody.source_root_frame_sha256() != selected_root_digest
        || custody.source_free_space_frame_sha256() != selected_free_digest
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let (next, manifest_bytes) = selected_blob_record(
        context,
        basis,
        selected_generation,
        base.manifest_record(),
        BlobRecordKind::DropSetManifestV3,
    )?;
    context = next;
    let Ok(BlobRecordV1::DropSetManifestV3(manifest)) =
        worth_store_physical_format::decode_blob_record(&manifest_bytes)
    else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    if !binding_matches(base, &manifest, &manifest_bytes, descriptor_record) {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = manifest.source_basis() else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    // A pending batch with a predecessor is judged by the same owner as a
    // completed one: its chain against the ordered history and the
    // checkpoint-source heads. Custody stays the joined source roster, which
    // the pending WAL admission consumes next.
    let chain = if base.predecessor().is_some() {
        let (next, chain) = historical_anchor::authenticate_pending_chain(
            context,
            basis,
            base,
            source,
            manifest.count(),
            operation_id,
        )?;
        context = next;
        Some(chain)
    } else {
        let (next, publication_bytes) = selected_blob_record(
            context,
            basis,
            selected_generation,
            source.publication_record(),
            BlobRecordKind::GenerationPublished,
        )?;
        context = next;
        if publication_bytes != source.publication().encode()
            || <[u8; 32]>::from(Sha256::digest(&publication_bytes))
                != source.publication_frame_sha256()
        {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        None
    };
    let source_routes = context.selection.page_facts().placements().to_vec();
    let predecessor_custody = match chain {
        Some(chain) => {
            let (next, custody) =
                historical_anchor::observe(context, basis, source, chain, &source_routes)?;
            context = next;
            Some(custody)
        }
        None => None,
    };
    context = selected::verify_initial(
        context,
        basis,
        base,
        &manifest,
        operation_id,
        &source_routes,
        Some(custody),
        Some(descriptor_record),
        match predecessor_custody.as_ref() {
            Some(predecessor) => predecessor.evidence(),
            None => ReleasedClosureEvidence::FirstPublication,
        },
    )?;
    basis.verified_drops.extend_from_slice(manifest.dropped());
    basis.verified_drops.sort_unstable();
    if basis
        .verified_drops
        .windows(2)
        .any(|pair| pair[0] == pair[1])
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    Ok(context)
}
