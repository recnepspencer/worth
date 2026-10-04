//! Released-generation descriptor preflight. This authenticates names and
//! selected source records; it does not authorize removal without a separate
//! complete selected-closure and prior-WAL-fate proof.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobReclaimDescriptorV2, BlobReclaimSourceBasisV1, BlobReclaimSourceKind, BlobRecordKind,
    BlobRecordV1, CurrentPhysicalRecordPlacement, DropSetManifestV3, PersistedRecordIdentity,
    SelectedRecordContentClass, BLOB_CONTROL_FRAME_MAX_BYTES,
};

use super::super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use super::super::historical_publication::{self, HistoricalFailure};
use super::record;

#[path = "released/closure_evidence.rs"]
mod closure_evidence;
#[path = "released/continuation.rs"]
mod continuation;
#[path = "released/external_edges.rs"]
mod external_edges;
#[path = "released/graph.rs"]
mod graph;
#[path = "released/head_predecessor.rs"]
mod head_predecessor;
#[path = "released/historical.rs"]
mod historical;
#[path = "released/historical_anchor.rs"]
mod historical_anchor;
#[path = "released/historical_predecessors.rs"]
mod historical_predecessors;
#[path = "released/historical_redo.rs"]
pub(super) mod historical_redo;
#[path = "released/postorder.rs"]
mod postorder;
#[path = "released/redo.rs"]
pub(super) mod redo;
#[path = "released/route_transcript.rs"]
mod route_transcript;
#[path = "released/selected.rs"]
mod selected;

pub(super) fn preflight(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    descriptor: BlobReclaimDescriptorV2,
    descriptor_record: PersistedRecordIdentity,
    operation_id: [u8; 32],
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    if descriptor.source_kind() != BlobReclaimSourceKind::ReleasedGeneration
        || descriptor.store() != context.authority.media.store_identity().bytes()
        || descriptor.manifest_record() == descriptor_record
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let (next, manifest_bytes) = selected_blob_record(
        context,
        basis,
        descriptor.source_root_generation(),
        descriptor.manifest_record(),
        BlobRecordKind::DropSetManifestV3,
    )?;
    context = next;
    let Ok(BlobRecordV1::DropSetManifestV3(manifest)) =
        worth_store_physical_format::decode_blob_record(&manifest_bytes)
    else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    if !binding_matches(descriptor, &manifest, &manifest_bytes, descriptor_record) {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = manifest.source_basis() else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    if let Some(predecessor) = descriptor.predecessor() {
        let (next, prior_bytes) = selected_blob_record(
            context,
            basis,
            descriptor.source_root_generation(),
            predecessor.descriptor_record(),
            BlobRecordKind::ReclaimDescriptorV2,
        )?;
        context = next;
        let Ok(BlobRecordV1::ReclaimDescriptorV2(prior)) =
            worth_store_physical_format::decode_blob_record(&prior_bytes)
        else {
            return Err(context.redo_block(basis.planning_counters(), None));
        };
        if <[u8; 32]>::from(Sha256::digest(&prior_bytes)) != predecessor.descriptor_frame_sha256()
            || prior.source_kind() != BlobReclaimSourceKind::ReleasedGeneration
            || prior.store() != descriptor.store()
            || prior.source_basis_digest() != descriptor.source_basis_digest()
            || prior.candidate_root_generation() != descriptor.source_root_generation()
            || prior.terminal()
            || prior
                .cumulative_dropped()
                .checked_add(u64::from(manifest.count()))
                != Some(descriptor.cumulative_dropped())
        {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
    } else {
        let (next, publication_bytes) = selected_blob_record(
            context,
            basis,
            descriptor.source_root_generation(),
            source.publication_record(),
            BlobRecordKind::GenerationPublished,
        )?;
        context = next;
        if publication_bytes != source.publication().encode()
            || <[u8; 32]>::from(Sha256::digest(&publication_bytes))
                != source.publication_frame_sha256()
            || manifest
                .dropped()
                .binary_search(&source.publication_record())
                .is_err()
        {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
    }
    // The first nonterminal batch has exactly one semantic effect: removal of
    // the authenticated publication. Later post-order batches require an
    // independently selected predecessor WAL/closure witness.
    if descriptor.predecessor().is_some()
        || descriptor.terminal()
        || manifest.dropped() != [source.publication_record()]
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let selected_generation = context
        .selection
        .root()
        .selected()
        .selector()
        .root_generation();
    let source_routes = if selected_generation == descriptor.source_root_generation() {
        context.selection.page_facts().placements().to_vec()
    } else if selected_generation >= descriptor.candidate_root_generation() {
        let (next, routes) = historical_publication::observe_all_routes(
            context,
            basis,
            descriptor.source_root_generation(),
            source.publication_record(),
        )?;
        context = next;
        routes
    } else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    context = selected::verify_initial(
        context,
        basis,
        descriptor,
        &manifest,
        operation_id,
        &source_routes,
        None,
        None,
        closure_evidence::ReleasedClosureEvidence::FirstPublication,
    )?;
    if selected_generation == descriptor.source_root_generation() {
        basis.verified_drops.push(source.publication_record());
        basis.verified_drops.sort_unstable();
        if basis
            .verified_drops
            .windows(2)
            .any(|pair| pair[0] == pair[1])
        {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
    } else {
        context = historical::verify_initial_result(
            context,
            basis,
            descriptor,
            &manifest,
            descriptor_record,
            source_routes.len() as u64,
        )?;
    }
    Ok(context)
}

fn binding_matches(
    descriptor: BlobReclaimDescriptorV2,
    manifest: &DropSetManifestV3,
    manifest_bytes: &[u8],
    descriptor_record: PersistedRecordIdentity,
) -> bool {
    descriptor.source_kind() == BlobReclaimSourceKind::ReleasedGeneration
        && manifest.source_kind() == BlobReclaimSourceKind::ReleasedGeneration
        && manifest.store() == descriptor.store()
        && manifest.reclaim_attempt() == descriptor.reclaim_attempt()
        && manifest.source_basis_digest() == descriptor.source_basis_digest()
        && manifest.count() == descriptor.manifest_count()
        && manifest.never_reserved_slot_generation() < descriptor.source_root_generation()
        && <[u8; 32]>::from(Sha256::digest(manifest_bytes)) == descriptor.manifest_frame_sha256()
        && manifest
            .dropped()
            .binary_search(&descriptor.manifest_record())
            .is_err()
        && manifest
            .dropped()
            .binary_search(&descriptor_record)
            .is_err()
        && (descriptor.predecessor().is_some()
            || descriptor.cumulative_dropped() == u64::from(manifest.count()))
}

fn selected_blob_record(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    generation: u64,
    identity: PersistedRecordIdentity,
    kind: BlobRecordKind,
) -> Result<(PlanningContext, Vec<u8>), crate::entry::PhysicalRecoveryOutcome> {
    let format = context.authority.record_format;
    historical_publication::observe(
        context,
        basis,
        generation,
        identity,
        |discovery, _, route, budget, trace, scratch| {
            if !matches!(route,
                Some(CurrentPhysicalRecordPlacement::Extent(extent))
                    if extent.content_class() == SelectedRecordContentClass::Blob(kind))
            {
                return Err(HistoricalFailure::Invalid);
            }
            record::read(
                discovery,
                format,
                route,
                identity,
                BLOB_CONTROL_FRAME_MAX_BYTES as u64,
                budget,
                trace,
                scratch,
            )
        },
    )
}

#[cfg(test)]
#[path = "released/head_anchor_tests.rs"]
mod head_anchor_tests;
#[cfg(test)]
#[path = "released/tests.rs"]
mod tests;
