//! Authenticated historical V3 result against its own addressed candidate.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{BlobReclaimSourceBasisV1, BlobRecordKind, BlobRecordV1};

use super::super::super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use super::super::historical_publication::{self, HistoricalFailure};
use super::closure_evidence::ReleasedClosureEvidence;
use super::{binding_matches, historical, historical_anchor, selected, selected_blob_record};
use crate::orchestration::planning::{
    page_observation::HistoricalDropEvidence, selected_source_inventory,
};
use crate::progression::verified_historical_release_transition;

pub(in crate::orchestration::planning::completion) fn verify_historical(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    evidence: &HistoricalDropEvidence,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    let descriptor = evidence.descriptor;
    let base = descriptor.base();
    let custody = descriptor.custody();
    let ordered_history = &*evidence.ordered_history;
    let mut matching = ordered_history
        .edges()
        .iter()
        .filter_map(|edge| match edge {
            worth_store_recovery_physics::VerifiedOrderedRootEdge::Released(edge)
                if edge.operation() == evidence.operation =>
            {
                Some(edge)
            }
            _ => None,
        });
    let (Some(ordered_edge), None) = (matching.next(), matching.next()) else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let selected_root_sha = <[u8; 32]>::from(Sha256::digest(
        context
            .selection
            .root()
            .selected()
            .manifest()
            .encode(context.authority.record_format),
    ));
    if base.store() != context.authority.media.store_identity().bytes()
        || base.source_root_generation().checked_add(1) != Some(base.candidate_root_generation())
        || ordered_edge.candidate_root_generation() != base.candidate_root_generation()
        || ordered_edge.descriptor_record() != evidence.descriptor_record
        || ordered_history.selected_root_frame_sha256() != selected_root_sha
        || custody.request().idempotency() != evidence.operation
        || base.manifest_record() == evidence.descriptor_record
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let format = context.authority.record_format;
    // The root's one entry, charged before it was read, pays for the
    // inventory under it too.
    let (next, historical_source) = historical_publication::observe_charged(
        context,
        basis,
        base.source_root_generation(),
        base.manifest_record(),
        |discovery, root, route, charge, budget, trace, _| {
            if route.is_none()
                || Sha256::digest(root.encode(format)).as_slice()
                    != custody.source_root_frame_sha256()
            {
                return Err(HistoricalFailure::Invalid);
            }
            let inventory = selected_source_inventory::observe_with_budget(
                discovery, root, format, charge, budget, trace,
            )?;
            if <[u8; 32]>::from(Sha256::digest(inventory.free_space.encode(format)))
                != custody.source_free_space_frame_sha256()
            {
                return Err(HistoricalFailure::Invalid);
            }
            Ok((root.clone(), inventory))
        },
    )?;
    context = next;
    let (next, manifest_bytes) = selected_blob_record(
        context,
        basis,
        base.source_root_generation(),
        base.manifest_record(),
        BlobRecordKind::DropSetManifestV3,
    )?;
    context = next;
    let Ok(BlobRecordV1::DropSetManifestV3(manifest)) =
        worth_store_physical_format::decode_blob_record(&manifest_bytes)
    else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    if manifest != evidence.manifest
        || !binding_matches(base, &manifest, &manifest_bytes, evidence.descriptor_record)
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = manifest.source_basis() else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let chain = if base.predecessor().is_some() {
        let (next, chain) = historical_anchor::authenticate_completed_chain(
            context,
            basis,
            base,
            source,
            manifest.count(),
            evidence.operation,
        )?;
        context = next;
        Some(chain)
    } else {
        let (next, publication_bytes) = selected_blob_record(
            context,
            basis,
            base.source_root_generation(),
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
    let (mut next, source_routes) = historical_publication::observe_all_routes(
        context,
        basis,
        base.source_root_generation(),
        base.manifest_record(),
    )?;
    let predecessor_custody = match chain {
        Some(chain) => {
            let (observed, custody) =
                historical_anchor::observe(next, basis, source, chain, &source_routes)?;
            next = observed;
            Some(custody)
        }
        None => None,
    };
    context = selected::verify_initial(
        next,
        basis,
        base,
        &manifest,
        evidence.operation,
        &source_routes,
        Some(custody),
        Some(evidence.descriptor_record),
        match predecessor_custody.as_ref() {
            Some(predecessor) => predecessor.evidence(),
            None => ReleasedClosureEvidence::RetainedHistory(&[]),
        },
    )?;
    let mut removed = manifest.dropped().to_vec();
    let Some(projection) = basis
        .redo
        .projections()
        .iter()
        .find(|projection| projection.operation() == evidence.operation)
    else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    if let worth_store_physical_format::PersistedPhysicalRecoveryOperation::DerivedDirectory {
        retirement: Some(retirement),
        ..
    } = projection.materialization().operation()
    {
        removed.extend_from_slice(retirement.dropped_records());
    }
    let projected = projection.materialization().placements().to_vec();
    removed.sort_unstable();
    let route_count = source_routes.len() as u64;
    let free_count = historical_source.1.free_entries.len() as u64;
    let segment_count = historical_source.1.segment_pages.len() as u64;
    let projected_count = projected.len() as u64;
    let scratch_bytes = route_count
        .checked_mul(
            (std::mem::size_of::<worth_store_physical_format::CurrentPhysicalRecordPlacement>()
                + 2 * std::mem::size_of::<crate::progression::RecoveryBaseImageAction>())
                as u64,
        )
        .and_then(|bytes| {
            free_count
                .checked_mul(
                    (std::mem::size_of::<worth_store_physical_format::RecordFreeSpaceManifestEntry>(
                    ) + 4 * std::mem::size_of::<usize>()) as u64,
                )
                .and_then(|extra| bytes.checked_add(extra))
        })
        .and_then(|bytes| {
            segment_count.checked_mul(
                (std::mem::size_of::<worth_store_physical_format::RecordSegmentPageManifestEntry>()
                    + 4 * std::mem::size_of::<usize>()) as u64,
            ).and_then(|extra| bytes.checked_add(extra))
        })
        .and_then(|bytes| {
            projected_count
                .checked_mul(std::mem::size_of::<
                    worth_store_physical_format::CurrentPhysicalRecordPlacement,
                >() as u64)
                .and_then(|extra| bytes.checked_add(extra))
        });
    let Some(scratch_bytes) = scratch_bytes else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    basis
        .observed_pages
        .historical_publication_peak_scratch_bytes = basis
        .observed_pages
        .historical_publication_peak_scratch_bytes
        .max(scratch_bytes);
    let (next, candidate) = historical_publication::observe_charged(
        context,
        basis,
        base.candidate_root_generation(),
        evidence.descriptor_record,
        |discovery, root, route, charge, budget, trace, _| {
            if route.is_none() {
                return Err(HistoricalFailure::Invalid);
            }
            let inventory = selected_source_inventory::observe_with_budget(
                discovery, root, format, charge, budget, trace,
            )?;
            Ok((root.clone(), inventory))
        },
    )?;
    context = next;
    let (next, candidate_routes) = historical_publication::observe_all_routes(
        context,
        basis,
        base.candidate_root_generation(),
        evidence.descriptor_record,
    )?;
    context = next;
    // Ordered history joined the replacement to its historical source bytes.
    let directory_replacement = ordered_edge.transition().directory_replacement();
    let mut matching = basis
        .observed_pages
        .ordered_releases
        .iter()
        .flatten()
        .filter(|release| release.operation == evidence.operation);
    let (Some(release), None) = (matching.next(), matching.next()) else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    if release.descriptor != descriptor
        || release.head_replay.source_root_frame_sha256() != ordered_edge.source_root_frame_sha256()
        || release.head_replay.result_root_frame_sha256() != ordered_edge.result_root_frame_sha256()
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let Ok((transition, input_scratch)) = verified_historical_release_transition(
        &historical_source.0,
        &historical_source.1,
        &source_routes,
        &candidate.0,
        &candidate.1,
        &candidate_routes,
        &removed,
        &projected,
        Some(release.head_replay.replay()),
        directory_replacement,
        format,
        context.limits.manifest_entries,
        context.limits.staging_bytes,
    ) else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    if ordered_edge.transition() != &transition
        || !ordered_history.selected_topology().matches_headers(
            context.selection.root().selected().manifest(),
            &basis.observed_pages.selected_source.free_space,
            format,
        )
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    basis
        .observed_pages
        .historical_publication_peak_scratch_bytes = basis
        .observed_pages
        .historical_publication_peak_scratch_bytes
        .max(
            scratch_bytes
                .saturating_add(input_scratch)
                .saturating_add(transition.scratch_bytes()),
        );
    context = historical::verify_v3_result(
        context,
        basis,
        descriptor,
        evidence.descriptor_record,
        &manifest,
        directory_replacement.is_some(),
    )?;
    basis
        .verified_historical_release_operations
        .push(evidence.operation);
    basis.verified_historical_release_sources.push((
        evidence.operation,
        historical_source.0,
        historical_source.1.free_space,
    ));
    Ok(context)
}
